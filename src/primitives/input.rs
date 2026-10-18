use crate::properties::Properties;
use crate::style::Style;

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Danger,
    Ghost,
    Link,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    pub properties: Properties,
    pub style: Style,
    pub text: String,
    pub variant: ButtonVariant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Search,
    Url,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    pub properties: Properties,
    pub style: Style,
    pub value: String,
    pub placeholder: String,
    pub input_type: InputType,
    pub max_length: Option<usize>,
    pub readonly: bool,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropdownOption {
    pub label: String,
    pub value: String,
    pub disabled: bool,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Checkbox {
    pub properties: Properties,
    pub style: Style,
    pub checked: bool,
    pub label: String,
    pub indeterminate: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dropdown {
    pub properties: Properties,
    pub style: Style,
    pub options: Vec<DropdownOption>,
    pub selected_index: Option<usize>,
    pub placeholder: String,
    pub searchable: bool,
    pub multi_select: bool,
}
