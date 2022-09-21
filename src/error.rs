/// Partition table error
#[derive(Debug)]
pub enum Error {
    CsvError(csv::Error),
    IoError(std::io::Error),
    InvalidPartitionTable,
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Self {
        Error::CsvError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}
