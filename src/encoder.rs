/// An Exponential-Golomb writer.
pub struct ExpGolombEncoder<'a> {
    bit_buf: BitBuffer<'a>,
}

impl<'a> ExpGolombEncoder<'a> {
    /// Create a new `ExpGolombEncoder`.
    ///
    /// `start` denotes the starting position in the first byte of `buf` and goes from 0 (first) to
    ///  7 (last). This function returns `None` if the buffer is empty or if `start` is  not within
    /// \[0, 7\].
    /// 
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombEncoder;
    /// let mut buf = [0u8; 1];
    /// // Write starting at the second bit
    /// let mut writer = ExpGolombEncoder::new(&mut buf, 1).unwrap();
    /// writer.put_unsigned(2).unwrap();
    /// writer.close();
    /// assert_eq!(buf[0], 0b00110000);
    /// ```
    #[inline]
    #[must_use]
    pub fn new(buf: &'a mut [u8], start: u32) -> Option<ExpGolombEncoder<'a>> {
        if buf.is_empty() || start > 7 {
            return None;
        }
        Some(ExpGolombEncoder {
            bit_buf: BitBuffer::new(buf, start),
        })
    }

    /// Encode a `u64` into the buffer. Returns `None` if the buffer is full.
    ///
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombEncoder;
    /// let mut buf = [0u8; 6];
    /// let mut writer = ExpGolombEncoder::new(&mut buf, 0).unwrap();
    /// for i in 0..=8 {
    ///     writer.put_unsigned(i).unwrap();
    /// }
    /// writer.close();
    /// 
    /// assert_eq!(
    ///     buf,
    ///     [0b10100110, 0b01000010, 0b10011000, 0b11100010, 0b00000100, 0b10000000]
    /// );
    /// ```
    /// 
    /// This function guards against out of bounds indexing by returning `None`:
    /// 
    /// ```
    /// # use exp_golomb::ExpGolombEncoder;
    /// let mut buf = [0u8; 1];
    /// let mut writer = ExpGolombEncoder::new(&mut buf, 0).unwrap();
    /// assert!(writer.put_unsigned(1).is_some());
    /// assert!(writer.put_unsigned(1).is_some());
    /// assert!(writer.put_unsigned(1).is_none());
    /// ```
    #[inline]
    #[must_use]
    pub fn put_unsigned(&mut self, value: u64) -> Option<()> {
        let xp1 = value.wrapping_add(1);

        let bytes = xp1.to_be_bytes();
        let lz = xp1.leading_zeros();
        let start = (lz / 8) as usize;
        let bit_start = lz - (lz / 8 * 8);

        let num_zeros = 64 - lz - 1;
        self.bit_buf.put_zeros(num_zeros);

        self.bit_buf.put_bytes(&bytes[start..], bit_start)
    }

    /// Write a single bit to the buffer. Returns `None` if the buffer is full.
    /// 
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombEncoder;
    /// let mut buf = [0u8; 1];
    /// let mut writer = ExpGolombEncoder::new(&mut buf, 6).unwrap();
    /// writer.put_bit(true).unwrap();
    /// writer.put_bit(false).unwrap();
    /// assert!(writer.put_bit(true).is_none());
    /// assert!(writer.put_bit(true).is_none());
    /// writer.close();
    /// assert_eq!(buf[0], 0b00000010);
    /// ```
    #[inline]
    #[must_use]
    pub fn put_bit(&mut self, value: bool) -> Option<()> {
        self.bit_buf.put_bit(value)
    }

    /// Consumes the `ExpGolombEncoder`, returning the bit position one past the last written bit.
    /// 
    /// # Examples
    ///
    /// ```
    /// # use exp_golomb::ExpGolombEncoder;
    /// let mut buf = [0u8; 1];
    /// let mut writer = ExpGolombEncoder::new(&mut buf, 2).unwrap();
    /// writer.put_unsigned(0).unwrap();
    /// assert_eq!(writer.close(), (0, 3));
    /// ```
    #[inline]
    pub fn close(self) -> (usize, u32) {
        (self.bit_buf.index, self.bit_buf.bit_pos)
    }
}

struct BitBuffer<'a> {
    buf: &'a mut [u8],
    index: usize,
    bit_pos: u32,
}

impl<'a> BitBuffer<'a> {
    #[inline]
    fn new(buf: &'a mut [u8], bit_pos: u32) -> BitBuffer<'a> {
        BitBuffer {
            buf,
            index: 0,
            bit_pos,
        }
    }

    #[inline]
    fn put_bit(&mut self, value: bool) -> Option<()> {
        *self.buf.get_mut(self.index)? |= (value as u8) << (7 - self.bit_pos);
        self.bit_pos += 1;
        if self.bit_pos >= 8 {
            self.bit_pos -= 8;
            self.index += 1;
        }
        Some(())
    }

    #[inline]
    fn put_zeros(&mut self, num_zeros: u32) -> Option<()> {
        // TODO: Suboptimal
        for _ in 0..num_zeros {
            self.put_bit(false)?;
        }
        Some(())
    }

    #[inline]
    #[must_use]
    fn put_bytes(&mut self, bytes: &[u8], mut start_pos: u32) -> Option<()> {
        for &byte in bytes {
            while start_pos < 8 {
                let data = ((byte as u32) << start_pos) >> self.bit_pos;
                *self.buf.get_mut(self.index)? |= data as u8;

                let shift_amount = 8 - u32::max(self.bit_pos, start_pos);
                self.bit_pos += shift_amount;
                if self.bit_pos >= 8 {
                    self.bit_pos -= 8;
                    self.index += 1;
                }

                start_pos += shift_amount;
            }
            start_pos -= 8;
        }
        Some(())
    }
}
