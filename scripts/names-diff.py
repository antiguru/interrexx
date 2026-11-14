#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Compares the names the pinned upstream `api/` headers declare with those of `include/`.

Both trees are read by clang alone: macro names come from the preprocessor (`-dM`), everything
else from the C++ syntax tree (`-ast-dump=json`). A declaration is recorded as one line: a
typedef with its underlying type, a record with its fields, an enum with its constants, a function
or method with its full signature, default arguments included (overloads differ only there).
Declarations that come from a header outside the two trees (system headers) are ignored.

The sets are diffed both ways. A difference is an error unless EXCEPTIONS below lists the line.
The text parsing this needs is limited to the compiler's output: nothing here reads a header's
text.

Usage: names-diff.py UPSTREAM_API_DIR OUR_INCLUDE_DIR
       names-diff.py --dump DIR       prints the lines of one tree
"""

import json
import os
import re
import subprocess
import sys
import tempfile

# Lines present in only one tree, each with its reason: (side, line prefix, reason). `side` is
# `upstream` (declared only upstream) or `ours`. A prefix covers every difference it starts, and an
# entry that covers none is reported as stale.
EXCEPTIONS = [
    (
        "upstream",
        "macro REXXAPIDEFS_INCLUDED",
        "include guard: the guard names are each header's own",
    ),
    ("upstream", "macro REXXAPITYPES_INCLUDED", "include guard"),
    ("upstream", "macro REXXPLATFORMAPIS_INCLUDED", "include guard"),
    ("upstream", "macro REXXPLATFORMDEFS_INCLUDED", "include guard"),
    ("upstream", "macro REXXSAA_INCLUDED", "include guard"),
    ("upstream", "macro RexxNativeInterface_Included", "include guard"),
    ("upstream", "macro ooRexxErrorCodes_Included", "include guard"),
    (
        "upstream",
        "macro __",
        "helper macros of the upstream entry-table machinery (`__functionstub`, `__methodstub`, "
        "`__cpp_function_proto`, `__cpp_method_proto`); include/ spells its own, "
        "under INTERREXX_",
    ),
    ("upstream", "macro oor__", "helper macros of the same machinery"),
    (
        "ours",
        "macro INTERREXX_",
        "include guards and helper macros of include/, all under one prefix",
    ),
    (
        "upstream",
        "field _RXFNCCAL_PARM::rxfnc_argv",
        "KNOWN DIFFERENCE: upstream declares the argument list non-const (PCONSTRXSTRING), "
        "include/ and the Rust layout declare it const. The entry goes when the field is fixed.",
    ),
    (
        "ours",
        "field _RXFNCCAL_PARM::rxfnc_argv",
        "the other side of the same difference",
    ),
]

# The translation unit: every public header, as a program would include them.
SOURCE = '#include "rexx.h"\n#include "oorexxapi.h"\n#include "oorexxerrors.h"\n'


def clang(args, directory, language="c++"):
    """Runs clang over SOURCE with the tree's include paths and answers its standard output."""
    flags = [
        "clang++",
        "-x",
        language,
        "-std=c++17",
        "-I",
        directory,
        "-I",
        os.path.join(directory, "platform/unix"),
    ]
    result = subprocess.run(
        flags + args + ["-"], input=SOURCE, capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        sys.exit(f"clang failed for {directory}:\n{result.stderr}")
    return result.stdout


def macro_names(directory):
    """The names of the macros the headers define, system macros excluded."""
    with_headers = clang(["-E", "-dM"], directory)
    # Macros of the compiler and of the system headers the API headers include.
    base = subprocess.run(
        ["clang++", "-x", "c++", "-std=c++17", "-E", "-dM", "-"],
        input="#include <stddef.h>\n#include <stdint.h>\n#include <stdbool.h>\n#include <limits.h>\n"
        "#include <stdlib.h>\n#include <string.h>\n#include <stdio.h>\n#include <wchar.h>\n",
        capture_output=True,
        text=True,
        check=True,
    ).stdout

    def names(text):
        return {line.split()[1].split("(")[0] for line in text.splitlines() if line}

    return names(with_headers) - names(base)


def strip(node):
    """The node without locations, ids and implementation bodies: what a signature compares."""
    if isinstance(node, dict):
        return {
            key: strip(value)
            for key, value in node.items()
            if key
            not in (
                "id",
                "loc",
                "range",
                "includedFrom",
                "isUsed",
                "isReferenced",
                "mangledName",
                "typeAliasDeclId",
                "previousDecl",
                "parentDeclContextId",
                "ownedTagDecl",
            )
        }
    if isinstance(node, list):
        return [strip(item) for item in node]
    return node


TYPEDEFS = {}


def qual(type_node):
    """A type in canonical form: every typedef name is replaced by its definition, so `PRXSTRING`
    and `RXSTRING *` agree. A swapped typedef at a use site is not seen here: the typedef lines
    compare each typedef's own definition, and the ABI dump measures each typedef and the type of
    every handler and export."""
    if not type_node:
        return ""
    text = type_node.get("qualType", "")
    # An anonymous type's spelling names the file and line of its declaration.
    text = re.sub(r"\(unnamed (struct|union) at [^)]*\)", r"(unnamed \1)", text)
    for _ in range(16):
        resolved = re.sub(
            r"\b[A-Za-z_]\w*\b", lambda m: TYPEDEFS.get(m.group(0), m.group(0)), text
        )
        if resolved == text:
            break
        text = resolved
    return text


def collect_typedefs(node):
    """Records every typedef of the translation unit, system headers included."""
    if node.get("kind") in ("TypedefDecl", "TypeAliasDecl") and "name" in node:
        under = node.get("type", {}).get("qualType", "")
        under = re.sub(r"\(unnamed (struct|union) at [^)]*\)", r"(unnamed \1)", under)
        TYPEDEFS[node["name"]] = f"({under})" if "(" in under else under
    for child in node.get("inner", []):
        collect_typedefs(child)


def signature(node):
    """A function or method: name, type, parameter types (names are not part of it), defaults."""
    parameters = []
    for child in node.get("inner", []):
        if child["kind"] == "ParmVarDecl":
            default = [
                json.dumps(strip(c), sort_keys=True) for c in child.get("inner", [])
            ]
            parameters.append(
                qual(child.get("type"))
                + (f" = {' '.join(default)}" if default else "")
            )
    flags = "".join(
        f" {key}"
        for key in ("constexpr", "virtual", "storageClass", "const")
        if node.get(key)
    )
    return f"{node.get('name', '')} {qual(node.get('type'))} ({', '.join(parameters)}){flags}"


class Walker:
    def __init__(self, directory):
        self.root = os.path.realpath(directory) + os.sep
        self.file = None
        self.lines = set()

    def track(self, node):
        """Follows the file the dumper is printing: it names a file only when it changes."""
        for place in (node.get("loc"), *(node.get("range") or {}).values()):
            if not place:
                continue
            for part in (place, place.get("spellingLoc"), place.get("expansionLoc")):
                if part and "file" in part:
                    self.file = part["file"]

    def ours(self):
        return self.file is not None and os.path.realpath(self.file).startswith(self.root)

    def walk(self, node, owner=""):
        self.track(node)
        kind = node.get("kind", "")
        here = self.ours()
        name = node.get("name")
        if kind in ("TranslationUnitDecl", "LinkageSpecDecl", "NamespaceDecl"):
            for child in node.get("inner", []):
                self.walk(child, owner)
            return
        if kind in ("CXXRecordDecl", "RecordDecl") and not node.get("isImplicit"):
            if here and node.get("completeDefinition"):
                label = name or f"<anonymous in {owner or 'file'}>"
                # `class` upstream and `struct` here differ only in the default access, not in a name.
                self.lines.add(f"record {owner}{label}")
                for child in node.get("inner", []):
                    self.walk(child, f"{owner}{label}::" if name else owner)
                return
            for child in node.get("inner", []):
                self.walk(child, owner)
            return
        if kind == "EnumDecl":
            if here and node.get("inner"):
                self.lines.add(f"enum {owner}{name or ''}")
                for child in node["inner"]:
                    self.track(child)
                    if child["kind"] == "EnumConstantDecl":
                        value = [
                            json.dumps(strip(c), sort_keys=True)
                            for c in child.get("inner", [])
                        ]
                        self.lines.add(
                            f"enumerator {owner}{name or ''}::{child['name']} {' '.join(value)}"
                        )
            else:
                for child in node.get("inner", []):
                    self.track(child)
            return
        if here and not node.get("isImplicit"):
            if kind == "TypedefDecl" or kind == "TypeAliasDecl":
                self.lines.add(f"typedef {owner}{name} {qual(node.get('type'))}")
            elif kind == "FieldDecl":
                self.lines.add(f"field {owner}{name or ''} {qual(node.get('type'))}")
            elif kind == "VarDecl":
                self.lines.add(f"variable {owner}{name} {qual(node.get('type'))}")
            elif kind in ("FunctionDecl", "CXXMethodDecl", "CXXConstructorDecl", "CXXDestructorDecl"):
                self.lines.add(f"{kind} {owner}{signature(node)}")
        for child in node.get("inner", []):
            self.walk(child, owner)


def declarations(directory):
    """The declaration lines of the headers under `directory`."""
    ast = json.loads(clang(["-fsyntax-only", "-Xclang", "-ast-dump=json"], directory))
    TYPEDEFS.clear()
    collect_typedefs(ast)
    walker = Walker(directory)
    walker.walk(ast)
    return walker.lines


def lines_of(directory):
    macros = {f"macro {name}" for name in macro_names(directory)}
    return macros | declarations(directory)


def main():
    if len(sys.argv) == 3 and sys.argv[1] == "--dump":
        print("\n".join(sorted(lines_of(sys.argv[2]))))
        return 0
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    upstream, ours = lines_of(sys.argv[1]), lines_of(sys.argv[2])
    failures = []
    used = set()
    for side, only in (("upstream", upstream - ours), ("ours", ours - upstream)):
        for line in sorted(only):
            covering = [
                index
                for index, (s, prefix, _) in enumerate(EXCEPTIONS)
                if s == side and line.startswith(prefix)
            ]
            used.update(covering)
            if not covering:
                failures.append(f"only {side}: {line}")
    for index, (side, prefix, _) in enumerate(EXCEPTIONS):
        if index not in used:
            failures.append(f"stale exception: {side} {prefix}")
    if failures:
        print("\n".join(failures))
        print(f"{len(failures)} difference(s)", file=sys.stderr)
        return 1
    print(f"names agree: {len(upstream)} upstream lines, {len(ours)} in include/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
