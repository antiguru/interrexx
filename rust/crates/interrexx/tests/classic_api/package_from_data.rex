t = .methodtester~new
name = "bug2108-package-from-data.cls"
data = "::class Bug2108Thing public" || .endOfLine || "::method hi" || .endOfLine || "  return 'hi'" || .endOfLine
p = t~TestLoadPackageFromData(name, data)
say p~isA(.Package) p~name
drop p
code = "return .Bug2108Thing~new~hi", "::requires '"name"'"
say .routine~new("before", code)~call
do 100
  x = .array~new(100)~fill(copies("x", 100))
end
drop x
say .routine~new("after", code)~call
q = t~TestLoadPackageFromData(name, "say 'second prologue'")
say q~name q~classes~items
q = t~TestLoadPackageFromData("other.cls", "say 'other prologue'; return 1")
say q~name
::requires 'METHODPackage.cls'
