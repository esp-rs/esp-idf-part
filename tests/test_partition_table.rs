use std::fs;

use esp_idf_part::{AppType, Error, PartitionTable, SubType, Type};

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

        let table = table.unwrap();
        let factory = table.find("factory");
        assert!(factory.is_some());
        let factory = factory.unwrap();
        assert_eq!(factory.ty(), Type::App);
        assert_eq!(factory.subtype(), SubType::App(AppType::Factory));
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
fn test_error_when_no_app_partition() -> Result<(), String> {
    let csv = fs::read_to_string("tests/data/err_no_app_partition.csv").unwrap();

    match PartitionTable::try_from_str(csv) {
        Err(Error::NoAppPartition) => Ok(()),
        result => Err(format!(
            "expected `Err(Error::NoAppPartition)`, found `{result:?}`"
        )),
    }
}

#[test]
fn test_error_when_multiple_factory_partitions() -> Result<(), String> {
    let csv = fs::read_to_string("tests/data/err_multiple_factory.csv").unwrap();

    match PartitionTable::try_from_str(csv) {
        Err(Error::MultipleFactoryPartitions) => Ok(()),
        result => Err(format!(
            "expected `Err(Error::MultipleFactoryPartitions)`, found `{result:?}`"
        )),
    }
}

#[test]
fn test_error_when_unaligned_app_partition() -> Result<(), String> {
    let csv = fs::read_to_string("tests/data/err_unaligned_app_partition.csv").unwrap();

    match PartitionTable::try_from_str(csv) {
        Err(Error::UnalignedPartition) => Ok(()),
        result => Err(format!(
            "expected `Err(Error::UnalignedPartition)`, found `{result:?}`"
        )),
    }
}

#[test]
fn test_error_when_duplicate_partition_names() -> Result<(), String> {
    let csv = fs::read_to_string("tests/data/err_conflicting_names.csv").unwrap();

    match PartitionTable::try_from_str(csv) {
        Err(Error::DuplicatePartitions(name)) if &name == "ota_0" => Ok(()),
        result => Err(format!(
            "expected `Err(Error::DuplicatePartitions(\"ota_0\"))`, found `{result:?}`"
        )),
    }
}

#[test]
fn test_error_when_partitions_overlapping() -> Result<(), String> {
    let csv = fs::read_to_string("tests/data/err_unaligned_app_partition.csv").unwrap();

    match PartitionTable::try_from_str(csv) {
        Err(Error::UnalignedPartition) => Ok(()),
        result => Err(format!(
            "expected `Err(Error::UnalignedPartition)`, found `{result:?}`"
        )),
    }
}

#[test]
fn test_empty_offsets_are_correctly_calculated() {
    let csv = fs::read_to_string("tests/data/partition_table_unit_test_two_ota.csv").unwrap();
    let table = PartitionTable::try_from(csv).unwrap();

    let partitions = table.partitions();
    let first = &partitions[0];

    let mut offset = 0x9000;

    assert_eq!(first.name(), "nvs");
    assert_eq!(first.offset(), offset);
    offset += first.size();

    for i in 1..partitions.len() {
        let next = &partitions[i];
        assert_eq!(next.offset(), offset);
        offset += next.size();
    }
}
