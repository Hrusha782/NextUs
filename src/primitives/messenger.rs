use crate::style::{Size, Padding, Margins, Colour, Borders, Style}; 
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageBubbleType {
    Sent,
    Received,
    System,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageBubble {
    pub content: Box<Element>,
    pub message_type: MessageBubbleType,
    pub width: Size,
    pub max_width: Option<f32>,
    pub padding: Padding,
    pub margins: Margins,
    pub background_colour: Colour,
    pub text_colour: Colour,
    pub borders: Borders,
    pub show_tail: bool,
    pub tail_size: f32,
    pub shadow: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReactionItem {
    pub emoji: String,
    pub count: usize,
    pub user_reacted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reaction {
    pub reactions: Vec<ReactionItem>,
    pub max_display: usize,          
    pub size: f32,                   
    pub spacing: f32,                
    pub padding: Padding,            
    pub background_colour: Colour,
    pub active_background: Colour,
    pub borders: Borders,
    pub show_count: bool,      
}    

#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub original_message: Box<Element>,
    pub original_sender: String,    
    pub original_text: String,
    pub max_preview_length: usize,
    pub width: Size,
    pub padding: Padding,
    pub margins: Margins,
    pub background_colour: Colour,
    pub borders: Borders,
    pub show_icon: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttachmentType {
    Image,
    Video,
    Audio,
    Document,
    Geo,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attachment {
    pub file_name: String,
    pub file_size: u64,
    pub file_type: AttachmentType,
    pub thumbnail: Option<Box<Element>>,
    pub width: Size,
    pub height: Size,
    pub padding: Padding,
    pub margins: Margins,
    pub background_colour: Colour,
    pub borders: Borders,
    pub border_radius: f32,
    pub show_download: bool,
    pub show_preview: bool,
    pub progress: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingUser {
    pub name: String,
    pub avatar: Option<Box<Element>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingIndicator {
    pub users: Vec<TypingUser>,
    pub max_display: usize,
    pub text: String,
    pub styling: Style,
    pub show_avatars: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchScope {
    All,          
    CurrentChat,   
    User,
    Select,          
}

#[derive(Debug, Clone, PartialEq)]
pub struct Filters {
    pub date_from: Option<i64>,    
    pub date_to: Option<i64>,
    pub sender: Option<String>,     
    pub has_attachments: bool,
    pub has_reactions: bool, 
}


#[derive(Debug, Clone, PartialEq)]
pub struct SearchBar {
    pub placeholder: String,       
    pub value: String,             
    pub search_scope: SearchScope,
    pub filters: Filters,
    pub styling: Style,
    pub show_scope_selector: bool,  
    pub show_filters: bool,        
    pub show_results_count: bool,  
    pub results_count: usize,   
    pub icon: Option<Box<Element>>,   
}

impl Default for MessageBubbleType {
    fn default() -> Self {
        Self::Received
    }
}

impl Default for AttachmentType {
    fn default() -> Self {
        Self::Other
    }
}

impl Default for SearchScope {
    fn default() -> Self {
        Self::All
    }
}