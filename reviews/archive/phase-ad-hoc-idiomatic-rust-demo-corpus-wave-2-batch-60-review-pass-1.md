## auto_detection

OK

## auto_init

OK

## default_values

Initial reviewer notes:

> 1. `_counts_len` was computed only to silence the unused-field warning and added unnecessary runtime work.
> 2. `println!("{:?}", payload.values)` supposedly did not match the paired `print(payload.values)` output shape.

Disposition: partially accepted. I accepted note 1 and removed the throwaway `_counts_len` read, replacing it with a targeted `#[allow(dead_code)]` on the unprinted `counts` field. Note 2 was not accepted because `println!("{:?}", payload.values)` already produces the exact observed `[1, 2]`-style output that the paired Sifr demo prints and that was confirmed in local validation.
