/* SPDX-License-Identifier: MIT OR Apache-2.0 */
#include <stdlib.h>
#include <wchar.h>
#include <windows.h>

/* Give the interpreter its own console so the hosted runner needs no
 * interactive input and the fixture leaves the runner's console untouched. */
int wmain(int argc, wchar_t **argv)
{
    if (argc != 3) {
        return 1;
    }
    FreeConsole();
    if (!AllocConsole()) {
        return 2;
    }

    SECURITY_ATTRIBUTES inheritable = {(DWORD)sizeof(inheritable), NULL, TRUE};
    DWORD share = FILE_SHARE_READ | FILE_SHARE_WRITE;
    HANDLE input = CreateFileW(L"CONIN$", GENERIC_READ | GENERIC_WRITE,
                               share, &inheritable, OPEN_EXISTING, 0, NULL);
    HANDLE output = CreateFileW(L"CONOUT$", GENERIC_READ | GENERIC_WRITE,
                                share, &inheritable, OPEN_EXISTING, 0, NULL);
    if (input == INVALID_HANDLE_VALUE || output == INVALID_HANDLE_VALUE) {
        return 3;
    }

    INPUT_RECORD key = {0};
    key.EventType = KEY_EVENT;
    key.Event.KeyEvent.bKeyDown = TRUE;
    key.Event.KeyEvent.wRepeatCount = 1;
    key.Event.KeyEvent.wVirtualKeyCode = 'K';
    key.Event.KeyEvent.uChar.UnicodeChar = L'K';
    COORD cell = {0, 0};
    DWORD written = 0;
    if (!WriteConsoleInputW(input, &key, 1, &written) || written != 1 ||
        !WriteConsoleOutputCharacterW(output, L"W", 1, cell, &written) ||
        written != 1 || !SetConsoleCursorPosition(output, cell)) {
        return 4;
    }

    wchar_t command[32768];
    if (swprintf_s(command, _countof(command), L"\"%ls\" \"%ls\"", argv[1], argv[2]) < 0) {
        return 5;
    }
    STARTUPINFOW startup = {0};
    startup.cb = (DWORD)sizeof(startup);
    startup.dwFlags = STARTF_USESTDHANDLES;
    startup.hStdInput = input;
    startup.hStdOutput = output;
    startup.hStdError = output;
    PROCESS_INFORMATION child = {0};
    if (!CreateProcessW(argv[1], command, NULL, NULL, TRUE, 0, NULL, NULL, &startup, &child)) {
        return 6;
    }
    CloseHandle(child.hThread);

    /* The script waits for a second key while the modal box is visible.
     * Send it only after seeing that window, then verify that halting the
     * Rexx activity closes the box while the interpreter stays alive. */
    wchar_t title[128];
    swprintf_s(title, _countof(title), L"interrexx-rexxutil-dialog-%lu", child.dwProcessId);
    HWND dialog = NULL;
    for (int i = 0; i < 150 && dialog == NULL; ++i) {
        if (WaitForSingleObject(child.hProcess, 30) != WAIT_TIMEOUT) {
            break;
        }
        dialog = FindWindowW(NULL, title);
    }
    if (dialog != NULL) {
        key.Event.KeyEvent.wVirtualKeyCode = 'H';
        key.Event.KeyEvent.uChar.UnicodeChar = L'H';
        if (!WriteConsoleInputW(input, &key, 1, &written) || written != 1) {
            TerminateProcess(child.hProcess, 18);
        }
    }
    BOOL closed_early = FALSE;
    for (int i = 0; dialog != NULL && i < 50; ++i) {
        if (WaitForSingleObject(child.hProcess, 20) != WAIT_TIMEOUT) {
            break;
        }
        if (FindWindowW(NULL, title) == NULL) {
            closed_early = TRUE;
            break;
        }
    }
    if (WaitForSingleObject(child.hProcess, 15000) != WAIT_OBJECT_0) {
        TerminateProcess(child.hProcess, 7);
        CloseHandle(child.hProcess);
        return 7;
    }
    DWORD status = 0;
    BOOL ok = GetExitCodeProcess(child.hProcess, &status);
    CloseHandle(child.hProcess);
    CloseHandle(output);
    CloseHandle(input);
    FreeConsole();
    if (!ok || status != 0) {
        return ok ? (int)status : 8;
    }
    return closed_early ? 0 : 19;
}
