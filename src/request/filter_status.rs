


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FilterStatus {
    Pass,
    FailFilterHeader,
    FailFilterQuery,
    FailFilterPath,
    FailFilterMethod,
    FailFilterScheme,
    FailFilterPort,
    FailFilterCustom,
}

