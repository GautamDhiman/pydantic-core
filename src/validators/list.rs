use pyo3::prelude::*;
use pyo3::types::PyDict;

use super::{SchemaValidator, ValResult, Validator};
use crate::errors::{err_val_error, ErrorKind, LocItem, ValError, ValLineError};
use crate::standalone_validators::validate_list;
use crate::utils::{dict_create, dict_get};

#[derive(Debug, Clone)]
pub struct ListValidator {
    item_validator: Option<Box<SchemaValidator>>,
    min_items: Option<usize>,
    max_items: Option<usize>,
}

impl Validator for ListValidator {
    fn is_match(type_: &str, _dict: &PyDict) -> bool {
        type_ == "list"
    }

    fn build(dict: &PyDict) -> PyResult<Self> {
        Ok(Self {
            item_validator: match dict_get!(dict, "items", &PyDict) {
                Some(d) => Some(Box::new(SchemaValidator::build(d)?)),
                None => None,
            },
            min_items: dict_get!(dict, "min_items", usize),
            max_items: dict_get!(dict, "max_items", usize),
        })
    }

    fn validate(&self, py: Python, obj: &PyAny) -> ValResult<PyObject> {
        let list = validate_list(py, obj)?;
        if let Some(min_length) = self.min_items {
            if list.len() < min_length {
                return err_val_error!(
                    py,
                    list,
                    kind = ErrorKind::ListTooShort,
                    context = Some(dict_create!(py, "min_length" => min_length))
                );
            }
        }
        if let Some(max_length) = self.max_items {
            if list.len() > max_length {
                return err_val_error!(
                    py,
                    list,
                    kind = ErrorKind::ListTooLong,
                    context = Some(dict_create!(py, "max_length" => max_length))
                );
            }
        }
        let mut output: Vec<PyObject> = Vec::with_capacity(list.len());
        let mut errors: Vec<ValLineError> = Vec::new();
        for (index, item) in list.iter().enumerate() {
            match self.item_validator {
                Some(ref validator) => match validator.validate(py, item) {
                    Ok(item) => output.push(item),
                    Err(ValError::LineErrors(line_errors)) => {
                        let loc = vec![LocItem::I(index)];
                        for err in line_errors {
                            errors.push(err.with_location(&loc));
                        }
                    }
                    Err(err) => return Err(err),
                },
                None => output.push(item.to_object(py)),
            }
        }

        if errors.is_empty() {
            Ok(output.to_object(py))
        } else {
            Err(ValError::LineErrors(errors))
        }
    }

    fn clone_dyn(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}