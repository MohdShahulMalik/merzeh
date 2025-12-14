use std::collections::HashMap;

#[derive(Debug, Default)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Number,
    Search,
    Date,
    Time,
    Checkbox,
    Radio,
    Select,
    Textarea,
}

impl InputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InputType::Text => "text",
            InputType::Password => "password",
            InputType::Number => "number",
            InputType::Search => "search",
            InputType::Date => "date",
            InputType::Time => "time",
            InputType::Checkbox => "checkbox",
            InputType::Radio => "radio",
            InputType::Select => "select",
            InputType::Textarea => "textarea",
        }
    }
}

#[derive(Debug, Default)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
    pub selected: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            selected: false,
        }
    }

    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

#[derive(Debug, Default)]
pub struct InputConfig {
    // Core Fields
    pub name: String,
    pub label: String,
    pub input_type: InputType,

    // Common Attributes
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub default_value: Option<String>,
    pub required: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub autofocus: bool,

    // Styling Things
    pub container_class: Option<String>,
    pub input_class: Option<String>,
    pub label_class: Option<String>,
    pub id: Option<String>,

    // Text Constraints
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub pattern: Option<String>,

    // Number Constraints
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,

    // Select/Radio Options
    pub options: Option<Vec<SelectOption>>,
    
    // Checkbox
    pub checked: bool,

    // Textarea
    pub rows: Option<u32>,
    pub cols: Option<u32>,

    // HTML Custom Data Attributes
    pub data_attributes: Option<HashMap<String, String>>,
}

impl InputConfig {
    pub fn text(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Text,
            ..Default::default()
        }
    }

    pub fn password(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Password,
            ..Default::default()
        }
    }

    pub fn number(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Number,
            ..Default::default()
        }
    }

    pub fn select(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Select,
            ..Default::default()
        }
    }

    pub fn search(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Search,
            ..Default::default()
        }
    }

    pub fn date(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Date,
            ..Default::default()
        }
    }

    pub fn time(name: impl Into<String>, label: impl Into<String>) -> Self {
                Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Time,
            ..Default::default()
        }
    }

    pub fn checkbox(name: impl Into<String>, label: impl Into<String>) -> Self {
                Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Checkbox,
            ..Default::default()
        }
    }

    pub fn radio(name: impl Into<String>, label: impl Into<String>) -> Self {
                Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Radio,
            ..Default::default()
        }
    }

    pub fn textarea(name: impl Into<String>, label: impl Into<String>) -> Self {
                Self {
            name: name.into(),
            label: label.into(),
            input_type: InputType::Textarea,
            ..Default::default()
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn help_text(mut self, text: impl Into<String>) -> Self {
        self.help_text = Some(text.into());
        self
    }

    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    pub fn autofocus(mut self) -> Self {
        self.autofocus = true;
        self
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.input_class = Some(class.into());
        self
    }

    pub fn container_class(mut self, class: impl Into<String>) -> Self {
        self.container_class = Some(class.into());
        self
    }

    pub fn label_class(mut self, class: impl Into<String>) -> Self {
        self.label_class = Some(class.into());
        self
    }

    pub fn min_length(mut self, min: u32) -> Self {
        self.min_length = Some(min);
        self
    }

    pub fn max_length(mut self, max: u32) -> Self {
        self.max_length = Some(max);
        self
    }

    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = Some(rows);
        self
    }

    pub fn cols(mut self, cols: u32) -> Self {
        self.cols = Some(cols);
        self
    }

    pub fn checked(mut self) -> Self {
        self.checked = true;
        self
    }

    pub fn get_id(&self) -> &str {
        self.id.as_deref().unwrap_or(&self.name)
    }

}
