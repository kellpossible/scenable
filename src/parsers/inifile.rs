/// Write the element out to an ini file in ini file format relevant
/// for that element.
pub trait ToIniFile {
    type Error;
    fn write_ini(&self, out: &mut impl std::io::Write) -> Result<(), Self::Error>;
}
