/// An Exponential-Golomb parser.
pub struct ExpGolombDecoder<'a> {
    iter: BitIterator<'a>,
}

impl<'a> ExpGolombDecoder<'a> {
    /// Create a new `ExpGolombDecoder`.
    ///
    /// `start` denotes the starting position in the first byte of `buf` and goes from 0 (first) to
    ///  7 (last). This function returns `None` if the buffer is empty or if `start` is  not within
    /// \[0, 7\].
    ///
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombDecoder;
    /// let data = [0b01000000];
    /// let mut reader = ExpGolombDecoder::new(&data, 0).unwrap();
    /// assert_eq!(reader.next_unsigned(), Some(1));
    /// ```
    ///
    /// Start at the second bit:
    ///
    /// ```
    /// # use exp_golomb::ExpGolombDecoder;
    /// // Same as above but `010` is shifted one place to the right
    /// let data = [0b00100000];
    /// let mut reader = ExpGolombDecoder::new(&data, 1).unwrap();
    /// assert_eq!(reader.next_unsigned(), Some(1));
    /// ```
    #[inline]
    #[must_use]
    pub fn new(buf: &'a [u8], start: u32) -> Option<ExpGolombDecoder<'a>> {
        if buf.is_empty() || start > 7 {
            return None;
        }
        Some(ExpGolombDecoder {
            iter: BitIterator::new(buf, start),
        })
    }

    /// Read the next bit (i.e, as a flag). Returns `None` if the end of the bitstream is reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use exp_golomb::ExpGolombDecoder;
    ///
    /// let data = [0b01010101];
    /// let mut reader = ExpGolombDecoder::new(&data, 4).unwrap();
    ///
    /// assert_eq!(reader.next_bit(), Some(0));
    /// assert_eq!(reader.next_bit(), Some(1));
    /// assert_eq!(reader.next_bit(), Some(0));
    /// assert_eq!(reader.next_bit(), Some(1));
    /// assert_eq!(reader.next_bit(), None);
    /// assert_eq!(reader.next_bit(), None);
    /// ```
    #[inline]
    pub fn next_bit(&mut self) -> Option<u8> {
        self.iter.next()
    }

    #[inline]
    fn count_leading_zeroes(&mut self) -> Option<u32> {
        let mut leading_zeros = 0;
        for bit in self.iter.by_ref() {
            if bit == 0 {
                leading_zeros += 1;
                if leading_zeros > u64::BITS {
                    return None;
                }
            } else {
                return Some(leading_zeros);
            }
        }
        None
    }

    /// Read the next Exp-Golomb value as an unsigned integer. Returns `None` if the end of the
    /// bitstream is reached before parsing is completed or if the coded value is exceeds the
    /// limits of a `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombDecoder;
    /// // 010               - 1
    /// // 00110             - 5
    /// // 00000000111111111 - 510
    /// // 00101             - 4
    /// // 01                - missing 1 more bit
    /// let data = [0b01000110, 0b00000000, 0b11111111, 0b10010101];
    ///
    /// let mut reader = ExpGolombDecoder::new(&data, 0).unwrap();
    /// assert_eq!(reader.next_unsigned(), Some(1));
    /// assert_eq!(reader.next_unsigned(), Some(5));
    /// assert_eq!(reader.next_unsigned(), Some(510));
    /// assert_eq!(reader.next_unsigned(), Some(4));
    /// assert_eq!(reader.next_unsigned(), None);
    /// assert_eq!(reader.next_unsigned(), None);
    /// ```
    ///
    /// The coded value is limited to 64 bits. Trying to parse larger values would return
    /// `None`.
    ///
    /// ```
    /// # use exp_golomb::ExpGolombDecoder;
    /// let data = [
    ///     0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
    ///     0b00000000, 0b00000001, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
    ///     0b11111111, 0b11111111, 0b11111111,
    /// ];
    /// let mut reader = ExpGolombDecoder::new(&data, 7).unwrap();
    /// assert_eq!(reader.next_unsigned(), Some(u64::MAX));
    ///
    /// // Attempt to parse a 65-bit number
    /// let mut reader = ExpGolombDecoder::new(&data, 6).unwrap();
    /// assert_eq!(reader.next_unsigned(), None);
    /// ```
    #[inline]
    #[must_use = "use `ExpGolombReader::skip_next` if the value is not needed"]
    pub fn next_unsigned(&mut self) -> Option<u64> {
        let mut lz = self.count_leading_zeroes()?;
        let x = 1u64.wrapping_shl(lz) - 1;
        let mut y = 0;

        if lz != 0 {
            for bit in self.iter.by_ref() {
                y <<= 1;
                y |= bit as u64;
                lz -= 1;
                if lz == 0 {
                    break;
                }
            }
            if lz != 0 {
                return None;
            }
        }
        Some(x + y)
    }

    /// Read the next Exp-Golomb value, interpreting it as a signed integer. Returns `None` if the
    /// end of the bitstream is reached before parsing is completed or if the coded value is
    /// exceeds the limits of a `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombDecoder;
    /// // Concatenated Wikipedia example:
    /// // https://en.wikipedia.org/wiki/Exponential-Golomb_coding#Extension_to_negative_numbers
    /// let data = [0b10100110, 0b01000010, 0b10011000, 0b11100010, 0b00000100, 0b10000000];
    ///
    /// let mut reader = ExpGolombDecoder::new(&data, 0).unwrap();
    /// assert_eq!(reader.next_signed(), Some(0));
    /// assert_eq!(reader.next_signed(), Some(1));
    /// assert_eq!(reader.next_signed(), Some(-1));
    /// assert_eq!(reader.next_signed(), Some(2));
    /// assert_eq!(reader.next_signed(), Some(-2));
    /// assert_eq!(reader.next_signed(), Some(3));
    /// assert_eq!(reader.next_signed(), Some(-3));
    /// assert_eq!(reader.next_signed(), Some(4));
    /// assert_eq!(reader.next_signed(), Some(-4));
    /// assert_eq!(reader.next_signed(), None);
    /// assert_eq!(reader.next_signed(), None);
    /// ```
    ///
    /// The coded value is limited to 64 bits. Trying to parse larger values would return
    /// `None`.
    ///
    /// ```
    /// # use exp_golomb::ExpGolombDecoder;
    /// let data = [
    ///     0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
    ///     0b00000000, 0b00000001, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
    ///     0b11111111, 0b11111111, 0b11111111,
    /// ];
    /// let mut reader = ExpGolombDecoder::new(&data, 7).unwrap();
    /// assert_eq!(reader.next_signed(), Some(i64::MIN));
    ///
    /// // Attempt to parse a 65-bit number
    /// let mut reader = ExpGolombDecoder::new(&data, 6).unwrap();
    /// assert_eq!(reader.next_signed(), None);
    /// ```
    #[inline]
    #[must_use = "use `ExpGolombReader::skip_next` if the value is not needed"]
    pub fn next_signed(&mut self) -> Option<i64> {
        self.next_unsigned().map(|k| {
            let factor = if k % 2 == 0 { -1 } else { 1 };
            factor * (k / 2 + k % 2) as i64
        })
    }

    /// Skip the next Exp-Golomb encoded value. Any parsing error at the end of the bitstream is
    /// ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombDecoder;
    /// let data = [0b01001001, 0b00110000];
    /// let mut reader = ExpGolombDecoder::new(&data, 0).unwrap();
    /// reader.skip_next();
    /// reader.skip_next();
    /// reader.skip_next();
    /// assert_eq!(reader.next_unsigned(), Some(2));
    /// reader.skip_next();
    /// assert_eq!(reader.next_unsigned(), None);
    /// reader.skip_next();
    /// assert_eq!(reader.next_unsigned(), None);
    /// ```
    #[inline]
    pub fn skip_next(&mut self) {
        if let Some(lz) = self.count_leading_zeroes() {
            self.iter.skip_bits(lz);
        }
    }
}

struct BitIterator<'a> {
    buf: &'a [u8],
    index: usize,
    bit_pos: u32,
}

impl<'a> BitIterator<'a> {
    #[inline]
    fn new(buf: &'a [u8], shift_sub: u32) -> BitIterator<'a> {
        Self {
            buf,
            index: 0,
            bit_pos: shift_sub,
        }
    }

    #[inline]
    fn skip_bits(&mut self, num_bits: u32) {
        let offset = self.bit_pos as usize + num_bits as usize;
        self.index = usize::min(self.buf.len(), self.index + offset / 8);
        self.bit_pos = (offset % 8) as u32;
    }
}

impl<'a> core::iter::Iterator for BitIterator<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let curr_byte = *self.buf.get(self.index)?;
        let shift = 7 - self.bit_pos;
        let bit = curr_byte & (1 << shift);

        self.bit_pos += 1;
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            // Increment only when the index has not reached the end of the buffer to prevent
            // wrap-around to a valid index which will make this function return `Some` after
            // signaling `None`
            if self.index < self.buf.len() {
                self.index += 1;
            }
        }

        Some(bit >> shift)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer() {
        assert!(ExpGolombDecoder::new(&[], 0).is_none());
    }

    #[test]
    fn start_bit_validity() {
        let data = [0b01000000];
        for i in 0..=7 {
            assert!(ExpGolombDecoder::new(&data, i).is_some());
        }
        assert!(ExpGolombDecoder::new(&data, 8).is_none());
    }

    #[test]
    fn shifted_data() {
        let data: [(&[u8], u32, Option<u64>); 9] = [
            (&[0b01000000], 0, Some(1)),
            (&[0b00100000], 1, Some(1)),
            (&[0b00010000], 2, Some(1)),
            (&[0b00001000], 3, Some(1)),
            (&[0b00000100], 4, Some(1)),
            (&[0b00000010], 5, Some(1)),
            (&[0b00000001], 6, None),
            (&[0b00000001, 0], 6, Some(1)),
            (&[0b00000000, 0b10000000], 7, Some(1)),
        ];

        for (buf, start, ans) in data {
            let mut reader = ExpGolombDecoder::new(buf, start).unwrap();
            let res = reader.next_unsigned();
            assert_eq!(res, ans);
        }
    }

    #[test]
    fn mix_next_unsigned_with_next_bit() {
        let data = [0b01010101];
        let mut reader = ExpGolombDecoder::new(&data, 0).unwrap();
        assert_eq!(reader.next_unsigned(), Some(1));
        assert_eq!(reader.next_bit(), Some(1));
        assert_eq!(reader.next_unsigned(), Some(1));
        assert_eq!(reader.next_bit(), Some(1));
    }
}
