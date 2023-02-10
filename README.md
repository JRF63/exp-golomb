# exp-golomb

Encoder/decoder for [Exponential-Golomb coding](https://en.wikipedia.org/wiki/Exponential-Golomb_coding).

Using the Wikipedia example:

|Number | Bit pattern |
|-------|-------------|
|0      | 1           |
|1      | 010         |
|2      | 011         |
|3      | 00100       |
|4      | 00101       |
|5      | 00110       |
|6      | 00111       |
|7      | 0001000     |
|8      | 000100      |

```rust
use exp_golomb::{ExpGolombDecoder, ExpGolombEncoder};

let mut buf = [0u8; 6];
let mut writer = ExpGolombEncoder::new(&mut buf, 0).unwrap();
for i in 0..=8 {
    writer.put_unsigned(i).unwrap();
}
writer.close();
    
assert_eq!(
    buf,
    [0b10100110, 0b01000010, 0b10011000, 0b11100010, 0b00000100, 0b10000000]
);

let mut reader = ExpGolombDecoder::new(&buf, 0).unwrap();
for i in 0..=8 {
    assert_eq!(reader.next_unsigned(), Some(i));
}
```