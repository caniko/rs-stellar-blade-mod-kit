struct ByteCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ByteCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn new_at(bytes: &'a [u8], offset: usize) -> Result<Self, UassetError> {
        if offset > bytes.len() {
            return Err(UassetError::Unsupported(format!(
                "cursor offset {offset} is outside file size {}",
                bytes.len()
            )));
        }
        Ok(Self { bytes, offset })
    }

    fn read_u32(&mut self) -> Result<u32, UassetError> {
        self.require(4)?;
        let value = u32::from_le_bytes(
            self.bytes[self.offset..self.offset + 4]
                .try_into()
                .expect("slice length checked"),
        );
        self.offset += 4;
        Ok(value)
    }

    fn read_i32(&mut self) -> Result<i32, UassetError> {
        self.require(4)?;
        let value = i32::from_le_bytes(
            self.bytes[self.offset..self.offset + 4]
                .try_into()
                .expect("slice length checked"),
        );
        self.offset += 4;
        Ok(value)
    }

    fn read_i64(&mut self) -> Result<i64, UassetError> {
        self.require(8)?;
        let value = i64::from_le_bytes(
            self.bytes[self.offset..self.offset + 8]
                .try_into()
                .expect("slice length checked"),
        );
        self.offset += 8;
        Ok(value)
    }

    fn read_fstring(&mut self) -> Result<String, UassetError> {
        let length = self.read_i32()?;
        if length == 0 {
            return Ok(String::new());
        }
        if length < 0 {
            return Err(UassetError::Unsupported(
                "UTF-16 FString parsing is not implemented".to_owned(),
            ));
        }
        let length = length as usize;
        self.require(length)?;
        let raw = &self.bytes[self.offset..self.offset + length];
        self.offset += length;
        let raw = raw.strip_suffix(&[0]).unwrap_or(raw);
        std::str::from_utf8(raw)
            .map(str::to_owned)
            .map_err(|error| UassetError::Unsupported(format!("invalid FString UTF-8: {error}")))
    }

    fn read_fname(&mut self, names: &[NameEntry]) -> Result<String, UassetError> {
        let index = self.read_i32()?;
        let number = self.read_i32()?;
        if index < 0 || index as usize >= names.len() {
            return Err(UassetError::Unsupported(format!(
                "FName index {} is outside name map count {}",
                index,
                names.len()
            )));
        }
        let base = names[index as usize].value.clone();
        if number == 0 {
            Ok(base)
        } else {
            Ok(format!("{base}_{number}"))
        }
    }

    fn skip(&mut self, len: usize) -> Result<(), UassetError> {
        self.require(len)?;
        self.offset += len;
        Ok(())
    }

    fn require(&self, len: usize) -> Result<(), UassetError> {
        if self.offset + len <= self.bytes.len() {
            Ok(())
        } else {
            Err(UassetError::Unsupported(format!(
                "asset ended unexpectedly at byte {} while reading {} bytes",
                self.offset, len
            )))
        }
    }
}
