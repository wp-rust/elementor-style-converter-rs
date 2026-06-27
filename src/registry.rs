use crate::converters::PropertyConverter;
use crate::expanders::ShorthandExpander;

/// Ordered list of converters. First match wins.
#[derive(Default)]
pub struct ConverterRegistry {
    converters: Vec<Box<dyn PropertyConverter>>,
}

impl ConverterRegistry {
    pub fn register(&mut self, converter: impl PropertyConverter + 'static) {
        self.converters.push(Box::new(converter));
    }

    pub fn all(&self) -> &[Box<dyn PropertyConverter>] {
        &self.converters
    }
}

/// Ordered list of shorthand expanders.
#[derive(Default)]
pub struct ExpanderRegistry {
    expanders: Vec<Box<dyn ShorthandExpander>>,
}

impl ExpanderRegistry {
    pub fn register(&mut self, expander: impl ShorthandExpander + 'static) {
        self.expanders.push(Box::new(expander));
    }

    pub fn all(&self) -> &[Box<dyn ShorthandExpander>] {
        &self.expanders
    }
}
