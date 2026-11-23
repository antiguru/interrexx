# SPDX-License-Identifier: MIT OR Apache-2.0
cmake_minimum_required(VERSION 3.21)
project(InterRexxOoDialog LANGUAGES CXX RC)

if(NOT MSVC OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
    message(FATAL_ERROR "ooDialog requires the x64 MSVC toolchain")
endif()

set(source "${OOREXX_SOURCE}/extensions/platform/windows/oodialog")
cmake_path(NORMAL_PATH source)
set(manifest "${OOREXX_SOURCE}/platform/windows/rexx64.exe.manifest")
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "${INTERREXX_SDK}/bin")
# Ensure multi-configuration generators use the same staged SDK bin directory.
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}")

add_library(rexx SHARED IMPORTED)
add_library(rexxapi SHARED IMPORTED)
foreach(lib IN ITEMS rexx rexxapi)
    set_target_properties(${lib} PROPERTIES
        IMPORTED_IMPLIB "${INTERREXX_SDK}/lib/${lib}.lib"
        IMPORTED_LOCATION "${INTERREXX_SDK}/bin/${lib}.dll")
endforeach()

include_directories("${INTERREXX_SDK}/include"
                    "${INTERREXX_SDK}/include/platform/windows"
                    "${source}")
string(TIMESTAMP copyright_year "%Y")
add_compile_definitions(_WIN32_WINNT=0x0600 NOMINMAX WIN32 _WINDOWS
                        _CRT_SECURE_NO_WARNINGS _CRT_NONSTDC_NO_DEPRECATE
                        "OOREXX_COPY_YEAR=\"2005-${copyright_year}\"")
set(shapes_version OOSHAPES_VER_MAJOR=4 OOSHAPES_VER_MINOR=2 OOSHAPES_VER_LEVEL=5
                   OOSHAPES_VER_BUILD=12940 OOSHAPES_COPYRIGHT_YEAR=2005-2025)

add_library(ooshapes SHARED "${source}/APICommon.cpp" "${source}/ooShapes.cpp"
                            "${source}/ooShapesAid.cpp" "${source}/ooShapesPE.cpp")
target_compile_definitions(ooshapes PRIVATE ${shapes_version})
target_link_libraries(ooshapes PRIVATE rexx rexxapi)

# The pinned tree uses these two ood*.cpp files in the launchers only.
file(GLOB dialog_sources "${source}/ood*.cpp")
list(REMOVE_ITEM dialog_sources "${source}/oodMain.cpp" "${source}/oodWinMain.cpp")
# On a case-insensitive filesystem the glob can also match ooDialog.cpp.
list(FILTER dialog_sources EXCLUDE REGEX "/ooDialog[.]cpp$")
add_library(oodialog SHARED ${dialog_sources} "${source}/APICommon.cpp"
                            "${source}/ooDialog.cpp" "${source}/oodialog.rc")
target_compile_definitions(oodialog PRIVATE ${shapes_version})
target_include_directories(oodialog PRIVATE "${OOREXX_SOURCE}/common"
                       "${OOREXX_SOURCE}/interpreter/platform/windows")
target_link_libraries(oodialog PRIVATE ooshapes rexx rexxapi
                      winmm comdlg32 comctl32 shlwapi oleacc uxtheme rpcrt4 propsys delayimp)

add_executable(oodialog_gui WIN32 "${source}/oodWinMain.cpp" "${source}/oodShared.cpp"
                                 "${source}/ooDialogWinMain.rc")
add_executable(oodialog_console "${source}/oodMain.cpp" "${source}/oodShared.cpp"
                                "${source}/ooDialogMain.rc")
set_target_properties(oodialog_gui PROPERTIES OUTPUT_NAME ooDialog)
set_target_properties(oodialog_console PROPERTIES OUTPUT_NAME ooDialog SUFFIX .com)
target_link_libraries(oodialog_gui PRIVATE rexx rexxapi shlwapi)
target_link_libraries(oodialog_console PRIVATE rexx rexxapi shell32 advapi32)
foreach(target IN ITEMS oodialog oodialog_gui oodialog_console)
    target_compile_definitions(${target} PRIVATE OOD_VER=4 OOD_REL=2 OOD_MOD=5 OOD_BLD=12940
        OOD_COPY_YEAR=2005-2025 "OOD_VER_STR=\"ooDialog 4.2.5.12940\""
        "OOD_BIN_NAME=\"$<TARGET_FILE_NAME:${target}>\"")
endforeach()
foreach(target IN ITEMS oodialog_gui oodialog_console)
    add_custom_command(TARGET ${target} POST_BUILD
        COMMAND "${CMAKE_MT}" -nologo -manifest "${manifest}"
                "-inputresource:$<TARGET_FILE:${target}>;#1"
                "-outputresource:$<TARGET_FILE:${target}>;#1"
        VERBATIM)
endforeach()
