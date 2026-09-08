# B24 changed-observer diagnostic result

INCONCLUSIVE for causation and production qualification; retained LLDB localization only.

| Process | Role | PID | State | Events |
| --- | --- | --- | --- | --- |
| 0 | warmup | 84536 | OBSERVED_UNDER_LLDB | 20 |
| 1 | observation | 84969 | OBSERVED_UNDER_LLDB | 20 |
| 2 | observation | 85183 | OBSERVED_UNDER_LLDB | 20 |
| 3 | observation | 85473 | OBSERVED_UNDER_LLDB | 20 |
| 4 | observation | 85744 | OBSERVED_UNDER_LLDB | 20 |
| 5 | observation | 85977 | OBSERVED_UNDER_LLDB | 20 |

Instruction partitions; inclusive overlapping loader/fixup/library details are in JSON.

| Partition | P0 | P1 | P2 | P3 | P4 | P5 | Range |
| --- | --- | --- | --- | --- | --- | --- | --- |
| measured_pre_first_stop_prefix | 292037 | 135741 | 146624 | 137212 | 126985 | 137486 | 165052 |
| pre_prepare | 3997202 | 3999923 | 3967261 | 3950941 | 3949824 | 3942367 | 57556 |
| fixups | 19358475 | 15123070 | 15129212 | 15139012 | 15130257 | 15297786 | 4235405 |
| library_initialization | 3809225 | 3655199 | 3650137 | 3643551 | 3679187 | 3778023 | 165674 |
| loader_other | 6641684 | 5609167 | 5541836 | 5557818 | 5622435 | 5595275 | 1099848 |
| remaining_preconstructor | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| constructor_to_main_unassigned | 390415 | 187130 | 187146 | 187109 | 182009 | 185661 | 208406 |
| cli_parse | 11656457 | 5532789 | 5698899 | 5656165 | 5518337 | 5550676 | 6138120 |
| config | 6994555 | 1910209 | 1656249 | 1884854 | 1642141 | 1904767 | 5352414 |
| discovery | 11592486 | 6704442 | 6532375 | 6674651 | 6467867 | 6699366 | 5124619 |
| file_read_1 | 263427 | 378555 | 237497 | 347813 | 227179 | 379314 | 152135 |
| file_check_1 | 7174040 | 2004751 | 1986091 | 1985532 | 1995101 | 1981696 | 5192344 |
| file_read_2 | 279152 | 392952 | 248304 | 358536 | 237847 | 390168 | 155105 |
| file_check_2 | 1967992 | 1803493 | 1803490 | 1786575 | 1786531 | 1785075 | 182917 |
| completion_residual | 423711 | 250778 | 501550 | 403693 | 372135 | 602397 | 351619 |

No debugger overhead is subtracted. Intervals are process-wide, including other threads.
Unknown work and the post-completion exit tail remain unassigned. Stability under LLDB
does not establish an uninstrumented fix or CV pass. B24/B27/full phase remain OPEN.
