#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Compares the names the pinned upstream `api/` headers declare with those of `include/`.

Both trees are read by clang alone, twice: as C++17 and as C11, so that what a header declares
only under `#ifndef __cplusplus` is compared as well. The lines of the two passes are united.
Macros come from the preprocessor (`-dM`): a macro is recorded with its name, parameters and
whitespace-normalised replacement text. Everything else comes from the syntax tree
(`-ast-dump=json`). A declaration is recorded as one line: a typedef with its underlying type, a
record with its fields, an enum with its constants, a function or method with its full signature
(storage class, `inline` and default arguments included, because overloads differ only there),
and a variable with its type and storage class. The members of an anonymous record and the
constants of an unnamed enum carry the position of their record or enum among its siblings, so a
member that moves is a difference. Declarations that come from a header outside the two trees
(system headers) are ignored. Every header of both trees is a unit of the comparison: the list of
headers is diffed, and each header a pass can include is read.

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

# Lines present in only one tree, each with its reason: (side, line prefix, reason). `side` is
# `upstream` (declared only upstream), `ours`, or `both` for a macro whose expansion differs, which
# shows as one line on each side. A prefix covers the lines that start with it followed by a space,
# an opening parenthesis or the end of the line, so a name never covers a longer one. An entry
# that covers no line is reported as stale.
GUARD = "include guard: the guard names are each header's own"
EXPANSION = "the same use with a differently written expansion"
STUBS = "helper macro of the upstream entry-table machinery; include/ spells its own"
CAST = (
    "ours casts each descriptor value to the parameter type, so a C method can take OSELF "
    "(decisions.md, the header fixes)"
)
THROW_API = (
    "the interface versions stay those of level 6.06: the Throw and ThrowObject entry points they "
    "announce belong to THROW, a level 6.07 feature, and refuse (decisions.md, the language level)"
)
EXCEPTIONS = [
    ("upstream", "macro REXXAPIDEFS_INCLUDED", GUARD),
    ("upstream", "macro REXXAPITYPES_INCLUDED", GUARD),
    ("upstream", "macro REXXPLATFORMAPIS_INCLUDED", GUARD),
    ("upstream", "macro REXXPLATFORMDEFS_INCLUDED", GUARD),
    ("upstream", "macro REXXSAA_INCLUDED", GUARD),
    ("upstream", "macro RexxNativeInterface_Included", GUARD),
    ("upstream", "macro ooRexxErrorCodes_Included", GUARD),
    ("upstream", "macro __cpp_function_proto", STUBS),
    ("upstream", "macro __cpp_method_proto", STUBS),
    ("upstream", "macro __functionstub", STUBS),
    ("upstream", "macro __methodstub", STUBS),
    ("upstream", "macro oor__adcl", STUBS),
    ("upstream", "macro oor__arg", STUBS),
    ("upstream", "macro oor__ret", STUBS),
    ("upstream", "macro oor__rytpe", STUBS),
    ("upstream", "macro oor__tdcl", STUBS),
    ("ours", "macro INTERREXX_C_LINKAGE", "helper macro of include/"),
    ("ours", "macro INTERREXX_DECLARE", "helper macro of include/"),
    ("ours", "macro INTERREXX_LIST", "helper macro of include/"),
    ("ours", "macro INTERREXX_READ", "helper macro of include/"),
    ("ours", "macro INTERREXX_OOREXXAPI_H", GUARD),
    ("ours", "macro INTERREXX_OOREXXERRORS_H", GUARD),
    ("ours", "macro INTERREXX_REXX_H", GUARD),
    ("ours", "macro INTERREXX_REXXAPIDEFS_H", GUARD),
    ("ours", "macro INTERREXX_REXXAPITYPES_H", GUARD),
    ("ours", "macro INTERREXX_REXXPLATFORMAPIS_H", GUARD),
    ("ours", "macro INTERREXX_REXXPLATFORMDEFS_H", GUARD),
    *[("both", f"macro RexxMethod{n}", CAST) for n in range(11)],
    *[("both", f"macro RexxRoutine{n}", CAST) for n in range(11)],
    (
        "both",
        "macro SET_REXX_VALUE",
        "ours sets the type code and the value; upstream's does not compile (decisions.md, the "
        "header fixes)",
    ),
    (
        "both",
        "macro SSIZE_MIN",
        "ours is the least ssize_t; upstream's is 0 (decisions.md, the header fixes)",
    ),
    (
        "both",
        "macro MAKERXSTRING",
        "ours is an expression, upstream's a block, so a use in an expression compiles",
    ),
    (
        "both",
        "macro REXX_LAST_METHOD",
        "ours omits a redundant cast of the null pointer, the initialiser is the same",
    ),
    (
        "both",
        "macro REXX_LAST_ROUTINE",
        "ours omits a redundant cast of the null pointer, the initialiser is the same",
    ),
    (
        "both",
        "macro RXSTRLEN",
        "ours casts both branches to size_t, which is the type the branches have upstream",
    ),
    (
        "both",
        "macro REXX_CLASSIC_ROUTINE_PROTOTYPE",
        "ours declares the function through the handler typedef, the same function type",
    ),
    (
        "both",
        "macro REXX_CURRENT_LANGUAGE_LEVEL",
        "interrexx implements level 6.06; 6.07 adds CATCH, FINALLY and THROW, which it does not have "
        "(decisions.md, the language level)",
    ),
    *[
        ("both", f"macro {name}", THROW_API)
        for name in (
            "CALL_INTERFACE_VERSION",
            "EXIT_INTERFACE_VERSION",
            "METHOD_INTERFACE_VERSION",
            "THREAD_INTERFACE_VERSION",
            "THREAD_INTERFACE_VERSION_5_3_0",
        )
    ],
    *[
        ("upstream", f"macro {name}", THROW_API)
        for name in (
            "CALL_INTERFACE_VERSION_5_3_0",
            "EXIT_INTERFACE_VERSION_5_3_0",
            "METHOD_INTERFACE_VERSION_5_3_0",
        )
    ],
]

# The headers of a tree, relative to its root, that are listed but not compiled: the platform
# headers of a platform the compiler here does not target.
NOT_COMPILED = ("platform/windows/",)


def headers(directory):
    """The relative paths of every header under `directory`, sorted."""
    found = []
    for base, _, files in os.walk(directory):
        for name in files:
            if name.endswith(".h"):
                found.append(os.path.relpath(os.path.join(base, name), directory))
    return sorted(found)


def source(directory):
    """The translation unit: every header the platform here compiles, as a program would include
    them. Each header is a unit of the comparison, not only those another one reaches."""
    return "".join(
        f'#include "{path}"\n'
        for path in headers(directory)
        if not path.startswith(NOT_COMPILED)
    )


# The two passes: (compiler, language flags).
PASSES = [("clang++", ["-x", "c++", "-std=c++17"]), ("clang", ["-x", "c", "-std=c11"])]


def clang(compiler, language, args, directory, text=None):
    """Runs clang over the tree's headers with its include paths and answers its standard output."""
    flags = [
        compiler,
        *language,
        "-I",
        directory,
        "-I",
        os.path.join(directory, "platform/unix"),
    ]
    result = subprocess.run(
        flags + args + ["-"],
        input=source(directory) if text is None else text,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        sys.exit(f"{compiler} failed for {directory}:\n{result.stderr}")
    return result.stdout


def macro_lines(compiler, language, directory):
    """The macros the headers define, as `macro NAME(PARAMS) EXPANSION` lines, system macros
    excluded. The expansion is the compiler's own, with every nested macro expanded and each
    parameter replaced by a placeholder, so two spellings of the same value agree (`1002` and a
    macro that expands to it) and a changed value or body does not."""
    text = source(directory)
    defined = clang(compiler, language, ["-E", "-dM"], directory)
    # Macros of the compiler and of the system headers the API headers include.
    base = clang(
        compiler,
        language,
        ["-E", "-dM"],
        directory,
        "#include <stddef.h>\n#include <stdint.h>\n#include <stdbool.h>\n#include <limits.h>\n"
        "#include <stdlib.h>\n#include <sys/types.h>\n#include <string.h>\n#include <stdio.h>\n#include <wchar.h>\n",
    )
    system = {m.group(1) for m in re.finditer(r"^#define (\w+)", base, re.M)}
    probes = []
    for match in re.finditer(r"^#define (\w+)(\(([^)]*)\))?", defined, re.M):
        name, params = match.group(1), match.group(3)
        if name in system:
            continue
        if params is None:
            probes.append(name)
        else:
            count = len([p for p in params.split(",") if p.strip()])
            # A variadic macro takes its variable arguments as one more placeholder.
            use = f"{name}({','.join(f'p{i}' for i in range(count))})"
            probes.append(use)
    # One line per macro: a string literal that names it, then its expansion.
    expanded = clang(
        compiler,
        language,
        ["-E", "-P"],
        directory,
        text + "".join(f'"{use}" {use}\n' for use in probes),
    )
    found = set()
    for line in expanded.splitlines():
        match = re.fullmatch(r'"(\w+(?:\([^)]*\))?)" ?(.*)', line)
        if match:
            body = " ".join(match.group(2).split())
            # Spaces beside a brace, bracket, parenthesis, comma, semicolon or `*` are spelling.
            body = re.sub(r" ?([{}()\[\],;*]) ?", r"\1", body)
            found.add(f"macro {match.group(1)} {body}".rstrip())
    return found


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

# The typedef that names an unnamed record or enum, by the id of the tag: `typedef struct {...} X;`
# declares the record X, however clang spells it.
TAG_NAMES = {}

# An anonymous type's spelling names the file and line of its declaration.
UNNAMED = re.compile(r"\(unnamed (struct|union|enum) at [^)]*\)")


def normalise(text):
    return UNNAMED.sub(r"(unnamed \1)", text)


def qual(type_node):
    """A type in canonical form: every typedef name is replaced by its definition, so `PRXSTRING`
    and `RXSTRING *` agree. A swapped typedef at a use site is not seen here: the typedef lines
    compare each typedef's own definition, and the ABI dump measures each typedef and the type of
    every handler and export."""
    if not type_node:
        return ""
    text = normalise(type_node.get("qualType", ""))
    for _ in range(16):
        resolved = re.sub(
            r"\b[A-Za-z_]\w*\b", lambda m: TYPEDEFS.get(m.group(0), m.group(0)), text
        )
        if resolved == text:
            break
        text = resolved
    return text


def owned_tags(node):
    """The ids of the tags a declaration declares inline, such as the struct of a typedef."""
    if isinstance(node, dict):
        owned = node.get("ownedTagDecl")
        if owned:
            yield owned["id"]
        for child in node.get("inner", []):
            yield from owned_tags(child)


def collect_typedefs(node):
    """Records every typedef of the translation unit, system headers included."""
    if node.get("kind") in ("TypedefDecl", "TypeAliasDecl") and "name" in node:
        under = normalise(node.get("type", {}).get("qualType", ""))
        TYPEDEFS[node["name"]] = f"({under})" if "(" in under else under
        for tag in owned_tags(node):
            TAG_NAMES[tag] = node["name"]
    for child in node.get("inner", []):
        collect_typedefs(child)


def flags(node, keys):
    """The flags of a declaration that are set, each by its name, a storage class by its value."""
    found = ""
    for key in keys:
        value = node.get(key)
        if value:
            found += f" {value if isinstance(value, str) else key}"
    return found


def signature(node):
    """A function or method: name, type, parameter types (names are not part of it), defaults.
    A method defined inside its class is inline whether or not it says so, so its `inline` is
    recorded only when the definition is outside the class."""
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
    keys = ["constexpr", "virtual", "storageClass", "const"]
    if node["kind"] == "FunctionDecl" or "parentDeclContextId" in node:
        keys.append("inline")
    return f"{node.get('name', '')} {qual(node.get('type'))} ({', '.join(parameters)})" + flags(
        node, keys
    )


class Walker:
    def __init__(self, directory):
        self.root = os.path.realpath(directory) + os.sep
        self.file = None
        self.lines = set()
        # How many anonymous records and unnamed enums each owner has had so far.
        self.anonymous = {}

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

    def position(self, owner, what):
        """The label of an anonymous record or unnamed enum: its position among its owner's."""
        count = self.anonymous.get((owner, what), 0) + 1
        self.anonymous[(owner, what)] = count
        return f"<anonymous {what} {count} in {owner or 'file'}>"

    def walk(self, node, owner=""):
        self.track(node)
        kind = node.get("kind", "")
        here = self.ours()
        name = node.get("name")
        if kind in ("CXXRecordDecl", "RecordDecl", "EnumDecl"):
            name = name or TAG_NAMES.get(node.get("id"))
        if kind in ("TranslationUnitDecl", "LinkageSpecDecl", "NamespaceDecl"):
            for child in node.get("inner", []):
                self.walk(child, owner)
            return
        if kind in ("CXXRecordDecl", "RecordDecl") and not node.get("isImplicit"):
            if here and node.get("completeDefinition"):
                label = name or self.position(owner, "record")
                # `class` upstream and `struct` here differ only in the default access, not in a name.
                self.lines.add(f"record {owner}{label}")
                for child in node.get("inner", []):
                    self.walk(child, f"{owner}{label}::")
                return
            for child in node.get("inner", []):
                self.walk(child, owner)
            return
        if kind == "EnumDecl":
            if here and node.get("inner"):
                label = name or self.position(owner, "enum")
                self.lines.add(f"enum {owner}{label}")
                for child in node["inner"]:
                    self.track(child)
                    if child["kind"] == "EnumConstantDecl":
                        value = [
                            json.dumps(strip(c), sort_keys=True)
                            for c in child.get("inner", [])
                        ]
                        self.lines.add(
                            f"enumerator {owner}{label}::{child['name']} {' '.join(value)}"
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
                self.lines.add(
                    f"variable {owner}{name} {qual(node.get('type'))}"
                    + flags(node, ("storageClass",))
                )
            elif kind in (
                "FunctionDecl",
                "CXXMethodDecl",
                "CXXConversionDecl",
                "CXXConstructorDecl",
                "CXXDestructorDecl",
            ):
                self.lines.add(f"{kind} {owner}{signature(node)}")
        for child in node.get("inner", []):
            self.walk(child, owner)


def declarations(compiler, language, directory):
    """The declaration lines of the headers under `directory`."""
    ast = json.loads(
        clang(compiler, language, ["-fsyntax-only", "-Xclang", "-ast-dump=json"], directory)
    )
    TYPEDEFS.clear()
    TAG_NAMES.clear()
    collect_typedefs(ast)
    walker = Walker(directory)
    walker.walk(ast)
    return walker.lines


def lines_of(directory):
    """Every line of the tree: its headers, then the macros and declarations of each pass."""
    lines = {f"header {path}" for path in headers(directory)}
    for compiler, language in PASSES:
        lines |= macro_lines(compiler, language, directory)
        lines |= declarations(compiler, language, directory)
    return lines


def covers(prefix, line):
    return line == prefix or (
        line.startswith(prefix) and line[len(prefix)] in " ("
    )


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
                if s in (side, "both") and covers(prefix, line)
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
