/* SPDX-License-Identifier: MIT OR Apache-2.0 */
#include <stdlib.h>
#include <wchar.h>
#include <windows.h>

/* Give the interpreter its own console so the hosted runner needs no
 * interactive input and the fixture leaves the runner's console untouched. */
int wmain(int argc, wchar_t **argv)
{
    BOOL key_bytes = argc == 4;
    if (argc != 3 && !key_bytes) {
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

    if (key_bytes) {
        UINT page = (UINT)wcstoul(argv[3], NULL, 10);
        if (!SetConsoleCP(page) || !SetConsoleOutputCP(page) ||
            !FlushConsoleInputBuffer(input)) {
            return 22;
        }

        /* Enhanced Up, F1, an accented letter, and ASCII sentinels. */
        INPUT_RECORD records[7] = {key, key, key, key, key, key, key};
        records[0].Event.KeyEvent.wVirtualKeyCode = VK_UP;
        records[0].Event.KeyEvent.wVirtualScanCode = 0x48;
        records[0].Event.KeyEvent.uChar.UnicodeChar = 0;
        records[0].Event.KeyEvent.dwControlKeyState = ENHANCED_KEY;
        records[1].Event.KeyEvent.wVirtualKeyCode = VK_F1;
        records[1].Event.KeyEvent.wVirtualScanCode = 0x3b;
        records[1].Event.KeyEvent.uChar.UnicodeChar = 0;
        records[2].Event.KeyEvent.wVirtualKeyCode = 'E';
        records[2].Event.KeyEvent.wVirtualScanCode = 0x12;
        records[2].Event.KeyEvent.uChar.UnicodeChar = L'\u00e9';
        for (int i = 3; i < 7; ++i) {
            records[i].Event.KeyEvent.wVirtualKeyCode = (WORD)('A' + i - 3);
            records[i].Event.KeyEvent.uChar.UnicodeChar = (wchar_t)('A' + i - 3);
        }
        if (!WriteConsoleInputW(input, records, _countof(records), &written) ||
            written != _countof(records)) {
            return 23;
        }
    }

    wchar_t command[32768];
    int length = key_bytes
        ? swprintf_s(command, _countof(command), L"\"%ls\" \"%ls\" \"%ls\"", argv[1], argv[2], argv[3])
        : swprintf_s(command, _countof(command), L"\"%ls\" \"%ls\"", argv[1], argv[2]);
    if (length < 0) {
        return 5;
    }
    STARTUPINFOW startup = {0};
    startup.cb = (DWORD)sizeof(startup);
    startup.dwFlags = STARTF_USESTDHANDLES;
    startup.hStdInput = input;
    startup.hStdOutput = output;
    startup.hStdError = output;
    PROCESS_INFORMATION child = {0};
    HANDLE job = CreateJobObjectW(NULL, NULL);
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION limit = {0};
    limit.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_JOB_MEMORY | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    limit.JobMemoryLimit = (SIZE_T)8 * 1024 * 1024 * 1024;
    if (job == NULL || !SetInformationJobObject(job, JobObjectExtendedLimitInformation,
                                               &limit, (DWORD)sizeof(limit))) {
        return 24;
    }
    if (!CreateProcessW(argv[1], command, NULL, NULL, TRUE, CREATE_SUSPENDED,
                        NULL, NULL, &startup, &child)) {
        return 6;
    }
    if (!AssignProcessToJobObject(job, child.hProcess) || ResumeThread(child.hThread) == (DWORD)-1) {
        TerminateProcess(child.hProcess, 25);
        return 25;
    }
    CloseHandle(child.hThread);
    if (key_bytes) {
        DWORD status = 0;
        BOOL finished = WaitForSingleObject(child.hProcess, 15000) == WAIT_OBJECT_0;
        BOOL ok = finished && GetExitCodeProcess(child.hProcess, &status);
        CloseHandle(child.hProcess);
        CloseHandle(job);
        return ok ? (int)status : 26;
    }

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
            TerminateProcess(child.hProcess, 21);
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
    CloseHandle(job);
    CloseHandle(output);
    CloseHandle(input);
    FreeConsole();
    if (!ok || status != 0) {
        return ok ? (int)status : 8;
    }
    return closed_early ? 0 : 19;
}
