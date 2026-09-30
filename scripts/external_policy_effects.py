"""Bounded external effects; native owners and generated-text delegation persist."""
from rust_policy_sites import code_tokens

EXTERNAL_RULES = {
    'external-owner-release': ('native-api-caller', 'owner-live-through-release',
                               'exclusive-terminal-release', 'consume-native-owner-once'),
    'external-owner-acquire': ('native-api-caller', 'inputs-live-through-acquisition',
                               'exclusive-owner-output', 'acquire-matched-native-owner'),
    'external-owner-transfer': ('native-api-caller', 'valid-unique-handle',
                                'exclusive-handle-transfer', 'transfer-native-owner-once'),
    'external-state-mutation': ('native-state-admission', 'state-live-through-mutation',
                                'scoped-state-mutation', 'retain-owner-after-state-change'),
    'external-output-write': ('native-api-caller', 'sized-output-live',
                              'exclusive-output-initialization', 'caller-owns-output-storage'),
}


def external_operation_family(site):
    # The source partition is immutable with respect to authored owner labels.
    from unsafe_policy_segments import segment_for
    if segment_for(site) != 'external':
        return None
    names = {t.value for t in code_tokens(site.text) if t.kind == 'ident'}
    # Mixed acquisition/cleanup owns both outcomes, rather than claiming a
    # terminal release of storage that has not yet been acquired.
    if names & {'OpenProcessToken', 'CreateFileW', 'CreateJobObjectW',
                'CreateToolhelp32Snapshot', 'OpenThread', 'pg_query_parse',
                'pg_query_normalize'}:
        return 'external-owner-acquire'
    if names & {'CloseHandle', 'pg_query_free_parse_result', 'pg_query_free_normalize_result'}:
        return 'external-owner-release'
    if 'from_raw_handle' in names:
        return 'external-owner-transfer'
    if names & {'fcntl', 'SetHandleInformation', 'kill', 'umask', 'sigaction',
                'SetConsoleCtrlHandler', 'SetInformationJobObject',
                'AssignProcessToJobObject', 'TerminateJobObject', 'ResumeThread',
                'MoveFileExW', 'CreateDirectoryW'}:
        return 'external-state-mutation'
    if names & {'statvfs', 'GetTokenInformation', 'GetDiskFreeSpaceExW',
                'GetFileInformationByHandle', 'Thread32First', 'Thread32Next'}:
        return 'external-output-write'
    return None
