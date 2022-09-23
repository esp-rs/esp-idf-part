use std::fs;

use esp_idf_part::{AppType, Error, PartitionTable, SubType, Type};

// `assert_matches!()` is an unstable feature, but it's useful so we'll emulate
// it for now.
macro_rules! assert_matches {
    ( $left:expr, $right:pat ) => {
        assert!(matches!($left, $right))
    };
}

#[test]
fn test_parse_bin() {
    let bin = fs::read("tests/data/single_factory_no_ota.bin").unwrap();
    let table = PartitionTable::try_from(bin).unwrap();
    let partitions = table.partitions();

    assert_eq!(partitions.len(), 3);
    assert_eq!(partitions[0].name(), "nvs");
    assert_eq!(partitions[1].name(), "phy_init");
    assert_eq!(partitions[2].name(), "factory");

    let bin = fs::read("tests/data/factory_app_two_ota.bin").unwrap();
    let table = PartitionTable::try_from(bin).unwrap();
    let partitions = table.partitions();

    assert_eq!(partitions.len(), 6);
    assert_eq!(partitions[3].name(), "factory");
    assert_eq!(partitions[3].ty(), Type::App);
    assert_eq!(partitions[3].subtype(), SubType::App(AppType::Factory));
    assert_eq!(partitions[3].offset(), 0x10000);
    assert_eq!(partitions[3].size(), 0x100000);
    assert_eq!(partitions[3].encrypted(), false);
}

#[test]
fn test_parse_csv() {
    let csv = fs::read_to_string("tests/data/single_factory_no_ota.csv").unwrap();
    let table = PartitionTable::try_from(csv).unwrap();
    let partitions = table.partitions();

    assert_eq!(partitions.len(), 3);
    assert_eq!(partitions[0].name(), "nvs");
    assert_eq!(partitions[1].name(), "phy_init");
    assert_eq!(partitions[2].name(), "factory");

    let csv = fs::read("tests/data/factory_app_two_ota.csv").unwrap();
    let table = PartitionTable::try_from(csv).unwrap();
    let partitions = table.partitions();

    assert_eq!(partitions.len(), 6);
    assert_eq!(partitions[3].name(), "factory");
    assert_eq!(partitions[3].ty(), Type::App);
    assert_eq!(partitions[3].subtype(), SubType::App(AppType::Factory));
    assert_eq!(partitions[3].offset(), 0x10000);
    assert_eq!(partitions[3].size(), 0x100000);
    assert_eq!(partitions[3].encrypted(), false);
}

#[test]
fn test_conversion_between_types() {
    let csv = fs::read_to_string("tests/data/single_factory_no_ota.csv").unwrap();
    let table_a = PartitionTable::try_from(csv).unwrap();
    let bin = table_a.to_bin().unwrap();
    let table_b = PartitionTable::try_from(bin).unwrap();

    assert_eq!(table_a, table_b);

    let bin = fs::read("tests/data/factory_app_two_ota.bin").unwrap();
    let table_a = PartitionTable::try_from(bin).unwrap();
    let csv = table_a.to_csv().unwrap();
    let table_b = PartitionTable::try_from(csv).unwrap();

    assert_eq!(table_a, table_b);
}

#[test]
fn test_esp_idf_unit_test_partition_tables() {
    let files = vec![
        "tests/data/partition_table_unit_test_app_2m.csv",
        "tests/data/partition_table_unit_test_app.csv",
        "tests/data/partition_table_unit_test_two_ota_2m.csv",
        "tests/data/partition_table_unit_test_two_ota.csv",
    ];

    for file in files {
        let csv = fs::read_to_string(file).expect("Unable to read partition table file");
        let table = PartitionTable::try_from(csv);
        assert!(table.is_ok());
    }
}

#[test]
fn test_circuitpython_partition_tables() {
    let files = vec![
        "tests/data/partitions-16MB-no-uf2.csv",
        "tests/data/partitions-16MB.csv",
        "tests/data/partitions-2MB-no-uf2.csv",
        "tests/data/partitions-4MB-no-uf2.csv",
        "tests/data/partitions-4MB.csv",
        "tests/data/partitions-8MB-no-uf2.csv",
        "tests/data/partitions-8MB.csv",
    ];

    for file in files {
        let csv = fs::read_to_string(file).unwrap();
        let table = PartitionTable::try_from(csv);
        assert!(table.is_ok());
    }
}

#[test]
fn test_error_when_no_app_partition() {
    let csv = fs::read_to_string("tests/data/err_no_app_partition.csv").unwrap();
    let table = PartitionTable::try_from(csv);

    let _expected: Result<PartitionTable, Error> = Err(Error::NoAppPartition);

    assert_matches!(table, _expected);
}

#[test]
fn test_error_when_multiple_factory_partitions() {
    let csv = fs::read_to_string("tests/data/err_multiple_factory.csv").unwrap();
    let table = PartitionTable::try_from(csv);

    let _expected: Result<PartitionTable, Error> = Err(Error::MultipleFactoryPartitions);

    assert_matches!(table, _expected);
}

#[test]
fn test_error_when_unaligned_app_partition() {
    let csv = fs::read_to_string("tests/data/err_unaligned_app_partition.csv").unwrap();
    let table = PartitionTable::try_from(csv);

    let _expected: Result<PartitionTable, Error> = Err(Error::UnalignedPartition);

    assert_matches!(table, _expected);
}

#[test]
fn test_error_when_duplicate_partition_names() {
    let csv = fs::read_to_string("tests/data/err_conflicting_names.csv").unwrap();
    let table = PartitionTable::try_from(csv);

    let _expected: Result<PartitionTable, Error> = Err(Error::DuplicatePartitions("ota_0".into()));

    assert_matches!(table, _expected);
}

#[test]
fn test_error_when_partitions_overlapping() {
    let csv = fs::read_to_string("tests/data/err_unaligned_app_partition.csv").unwrap();
    let table = PartitionTable::try_from(csv);

    let _expected: Result<PartitionTable, Error> =
        Err(Error::OverlappingPartitions("ota_0".into(), "ota_1".into()));

    assert_matches!(table, _expected);
}
