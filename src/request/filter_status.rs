


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FilterError {
    Pass,
    FailFilterHeader,
    FailFilterQuery,
    FailFilterPath,
    FailFilterMethod,
    FailFilterScheme,
    FailFilterPort,
    FailFilterCustom,
}

