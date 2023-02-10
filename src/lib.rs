#![warn(missing_docs)]
//! TODO

/// An Exponential-Golomb parser.
pub struct ExpGolombReader<'a> {
    iter: BitIterator<'a>,
}

impl<'a> ExpGolombReader<'a> {
    /// Create a new `ExpGolomb` reader.
    ///
    /// `start` denotes the starting position in the first byte of `buf` and goes from 0 (first) to
    ///  7 (last). This function returns `None` if the buffer is empty or if `start` is  not within
    /// \[0, 7\].
    ///
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombReader;
    /// let data = [0b01000000];
    /// let mut exp_golomb_reader = ExpGolombReader::new(&data, 0).unwrap();
    /// assert_eq!(exp_golomb_reader.next_unsigned(), Some(1));
    /// ```
    ///
    /// Start at the second bit:
    ///
    /// ```
    /// # use exp_golomb::ExpGolombReader;
    /// // Same as above but `010` is shifted one place to the right
    /// let data = [0b00100000];
    /// let mut exp_golomb_reader = ExpGolombReader::new(&data, 1).unwrap();
    /// assert_eq!(exp_golomb_reader.next_unsigned(), Some(1));
    /// ```
    #[inline]
    pub fn new(buf: &'a [u8], start: u8) -> Option<ExpGolombReader<'a>> {
        if buf.is_empty() || start > 7 {
            return None;
        }
        BitIterator::new(buf, start).map(|iter| ExpGolombReader { iter })
    }

    /// Read the next bit (i.e, as a flag). Returns `None` if the end of the bitstream is reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use exp_golomb::ExpGolombReader;
    ///
    /// let data = [0b01010101];
    /// let mut exp_golomb_reader = ExpGolombReader::new(&data, 4).unwrap();
    ///
    /// assert_eq!(exp_golomb_reader.next_bit(), Some(0));
    /// assert_eq!(exp_golomb_reader.next_bit(), Some(1));
    /// assert_eq!(exp_golomb_reader.next_bit(), Some(0));
    /// assert_eq!(exp_golomb_reader.next_bit(), Some(1));
    /// assert_eq!(exp_golomb_reader.next_bit(), None);
    /// assert_eq!(exp_golomb_reader.next_bit(), None);
    /// ```
    #[inline]
    pub fn next_bit(&mut self) -> Option<u8> {
        self.iter.next()
    }

    #[inline]
    fn count_leading_zeroes(&mut self) -> Option<usize> {
        let mut leading_zeros: usize = 0;
        while let Some(bit) = self.iter.next() {
            if bit == 0 {
                leading_zeros += 1;
            } else {
                return Some(leading_zeros);
            }
        }
        None
    }

    /// Read the next Exp-Golomb value as an unsigned integer. Returns `None` if the end of the
    /// bitstream is reached before parsing is completed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombReader;
    /// // 010               - 1
    /// // 00110             - 5
    /// // 00000000111111111 - 510
    /// // 00101             - 4
    /// // 01                - missing 1 more bit
    /// let data = [0b01000110, 0b00000000, 0b11111111, 0b10010101];
    /// 
    /// let mut exp_golomb_reader = ExpGolombReader::new(&data, 0).unwrap();
    /// assert_eq!(exp_golomb_reader.next_unsigned(), Some(1));
    /// assert_eq!(exp_golomb_reader.next_unsigned(), Some(5));
    /// assert_eq!(exp_golomb_reader.next_unsigned(), Some(510));
    /// assert_eq!(exp_golomb_reader.next_unsigned(), Some(4));
    /// assert_eq!(exp_golomb_reader.next_unsigned(), None);
    /// assert_eq!(exp_golomb_reader.next_unsigned(), None);
    /// ```
    #[inline]
    #[must_use = "use `ExpGolombReader::skip_next` if the value is not needed"]
    pub fn next_unsigned(&mut self) -> Option<u64> {
        let mut lz = self.count_leading_zeroes()?;
        let x = (1 << lz) - 1;
        let mut y = 0;

        if lz != 0 {
            while let Some(bit) = self.iter.next() {
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
        return Some(x + y);
    }

    /// Read the next Exp-Golomb value, interpreting it as a signed integer. Returns `None` if the
    /// end of the bitstream is reached before parsing is completed.
    /// 
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombReader;
    /// // Concatenated Wikipedia example:
    /// // https://en.wikipedia.org/wiki/Exponential-Golomb_coding#Extension_to_negative_numbers
    /// let data = [0b10100110, 0b01000010, 0b10011000, 0b11100010, 0b00000100, 0b10000000];
    /// 
    /// let mut exp_golomb_reader = ExpGolombReader::new(&data, 0).unwrap();
    /// assert_eq!(exp_golomb_reader.next_signed(), Some(0));
    /// assert_eq!(exp_golomb_reader.next_signed(), Some(1));
    /// assert_eq!(exp_golomb_reader.next_signed(), Some(-1));
    /// assert_eq!(exp_golomb_reader.next_signed(), Some(2));
    /// assert_eq!(exp_golomb_reader.next_signed(), Some(-2));
    /// assert_eq!(exp_golomb_reader.next_signed(), Some(3));
    /// assert_eq!(exp_golomb_reader.next_signed(), Some(-3));
    /// assert_eq!(exp_golomb_reader.next_signed(), Some(4));
    /// assert_eq!(exp_golomb_reader.next_signed(), Some(-4));
    /// assert_eq!(exp_golomb_reader.next_signed(), None);
    /// assert_eq!(exp_golomb_reader.next_signed(), None);
    /// ```
    #[inline]
    #[must_use = "use `ExpGolombReader::skip_next` if the value is not needed"]
    pub fn next_signed(&mut self) -> Option<i64> {
        self.next_unsigned()
            .map(|k| (-1i64).pow((k + 1) as u32) * (k / 2 + k % 2) as i64)
    }

    /// Skip the next Exp-Golomb encoded value. Returns `None` if the end of the bitstream is
    /// reached before parsing is completed.
    /// 
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombReader;
    /// let data = [0b01001001, 0b00110000];
    /// let mut exp_golomb_reader = ExpGolombReader::new(&data, 0).unwrap();
    /// assert_eq!(exp_golomb_reader.skip_next(), Some(()));
    /// assert_eq!(exp_golomb_reader.skip_next(), Some(()));
    /// assert_eq!(exp_golomb_reader.skip_next(), Some(()));
    /// assert_eq!(exp_golomb_reader.next_unsigned(), Some((2)));
    /// assert_eq!(exp_golomb_reader.skip_next(), None);
    /// assert_eq!(exp_golomb_reader.skip_next(), None);
    /// ```
    #[inline]
    pub fn skip_next(&mut self) -> Option<()> {
        let lz = self.count_leading_zeroes()?;
        // TODO: This could be more optimized
        for _ in 0..lz {
            self.iter.next()?;
        }
        Some(())
    }
}

struct BitIterator<'a> {
    buf: &'a [u8],
    index: usize,
    shift_sub: u8,
}

impl<'a> BitIterator<'a> {
    #[inline]
    fn new(buf: &'a [u8], shift_sub: u8) -> Option<Self> {
        Some(Self {
            buf,
            index: 0,
            shift_sub,
        })
    }
}

impl<'a> std::iter::Iterator for BitIterator<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let curr_byte = *self.buf.get(self.index)?;
        let shift = 7 - self.shift_sub;
        let bit = curr_byte & (1 << shift);

        self.shift_sub += 1;
        if self.shift_sub == 8 {
            self.shift_sub = 0;
            // Increment only when the index has not reached the end of the buffer
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
        assert!(ExpGolombReader::new(&[], 0).is_none());
    }

    #[test]
    fn start_bit_validity() {
        let data = [0b01000000];
        for i in 0..=7 {
            assert!(ExpGolombReader::new(&data, i).is_some());
        }
        assert!(ExpGolombReader::new(&data, 8).is_none());
    }

    #[test]
    fn shifted_data() {
        let data: Vec<(&[u8], u8, Option<u64>)> = vec![
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
            let mut exp_golomb = ExpGolombReader::new(buf, start).unwrap();
            let res = exp_golomb.next_unsigned();
            assert_eq!(res, ans);
        }
    }
}
