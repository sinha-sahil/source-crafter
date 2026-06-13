use serde::{Deserialize, Serialize};

/// Layout direction for a region
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Direction {
    #[serde(rename = "vertical")]
    Vertical,
    #[serde(rename = "horizontal")]
    Horizontal,
    #[serde(rename = "grid")]
    Grid,
}

/// Content justification within a region
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Justify {
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "end")]
    End,
    #[serde(rename = "space-between")]
    SpaceBetween,
}

/// Standard event types mapped to library-specific prop names by the adapter
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum EventType {
    #[serde(rename = "click")]
    Click,
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "change")]
    Change,
    #[serde(rename = "submit")]
    Submit,
    #[serde(rename = "focus")]
    Focus,
    #[serde(rename = "blur")]
    Blur,
}

/// Built-in action types the engine can execute
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ActionType {
    #[serde(rename = "show")]
    Show,
    #[serde(rename = "hide")]
    Hide,
    #[serde(rename = "toggle")]
    Toggle,
    #[serde(rename = "navigate")]
    Navigate,
    #[serde(rename = "back")]
    Back,
    #[serde(rename = "notify")]
    Notify,
    #[serde(rename = "scroll-to")]
    ScrollTo,
    #[serde(rename = "focus")]
    Focus,
    #[serde(rename = "enable")]
    Enable,
    #[serde(rename = "disable")]
    Disable,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "reset")]
    Reset,
    #[serde(rename = "fetch")]
    Fetch,
    #[serde(rename = "refresh")]
    Refresh,
    #[serde(rename = "update-region")]
    UpdateRegion,
    #[serde(rename = "update-props")]
    UpdateProps,
    #[serde(rename = "multiple")]
    Multiple,
    #[serde(rename = "handler")]
    Handler,
}

/// Top-level parsed YAML template
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageTree {
    pub page: String,
    pub regions: PageTreeRegions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleTokens>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<DataDeclarations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactions: Option<Vec<Interaction>>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageTreeRegions {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, Region>,
}

/// A named layout container that holds components or sub-regions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Region {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<Justify>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<f64>,
    #[serde(rename = "collapse-below")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapse_below: Option<f64>,
    #[serde(rename = "stack-below")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_below: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<RegionRegions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleBlock>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegionRegions {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, Region>,
}

/// A renderable UI component with type, props, and optional layout/style
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Component {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Props>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<f64>,
    #[serde(rename = "colSpan")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_span: Option<f64>,
    #[serde(rename = "rowSpan")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_span: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleBlock>,
}

/// Maps a component event to a built-in action or handler
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Interaction {
    pub on: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#do: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// An action the engine executes (built-in or returned by handler)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Action {
    pub r#type: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Props>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<Action>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ActionData>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActionData {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Layout-only view of a region — no component props
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayoutNode {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub direction: Direction,
    pub gap: f64,
    pub justify: Justify,
    pub span: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<f64>,
    pub layer: f64,
    #[serde(rename = "collapseBelow")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapse_below: Option<f64>,
    #[serde(rename = "stackBelow")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_below: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleBlock>,
    #[serde(rename = "childRegions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_regions: Option<Vec<LayoutNode>>,
    #[serde(rename = "childSlots")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_slots: Option<Vec<ComponentSlotRef>>,
}

/// Reference to a component in the flat component list
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComponentSlotRef {
    pub index: f64,
    pub span: f64,
    #[serde(rename = "colSpan")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_span: Option<f64>,
    #[serde(rename = "rowSpan")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_span: Option<f64>,
}

/// A component extracted from the tree with its region path
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComponentEntry {
    pub component: Component,
    #[serde(rename = "regionPath")]
    pub region_path: String,
}

/// Result of splitting a PageTree
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SplitResult {
    pub layout: Vec<LayoutNode>,
    pub components: Vec<ComponentEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<StyleTokens>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<SplitResultData>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SplitResultData {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, serde_json::Value>,
}

/// A layout container with CSS properties — output of the arranger
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContainerNode {
    pub id: String,
    #[serde(rename = "layoutType")]
    pub layout_type: LayoutTypeEnum,
    pub direction: DirectionEnum,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<Justify>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<f64>,
    #[serde(rename = "collapseBelow")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapse_below: Option<f64>,
    #[serde(rename = "stackBelow")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_below: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ContainerChild>>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LayoutTypeEnum {
    #[serde(rename = "flex")]
    Flex,
    #[serde(rename = "grid")]
    Grid,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DirectionEnum {
    #[serde(rename = "row")]
    Row,
    #[serde(rename = "column")]
    Column,
}

/// Either a nested container or a component slot
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContainerChild {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ContainerNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<ComponentSlot>,
}

/// A placeholder in the container tree for a component
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComponentSlot {
    #[serde(rename = "componentId")]
    pub component_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<f64>,
    #[serde(rename = "colSpan")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_span: Option<f64>,
    #[serde(rename = "rowSpan")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_span: Option<f64>,
}

/// A diff describing a layout change from resize recalculation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContainerUpdate {
    pub r#type: TypeEnum,
    #[serde(rename = "containerId")]
    pub container_id: String,
    #[serde(rename = "newDirection")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_direction: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TypeEnum {
    #[serde(rename = "hide")]
    Hide,
    #[serde(rename = "show")]
    Show,
    #[serde(rename = "change-direction")]
    ChangeDirection,
    #[serde(rename = "update-spans")]
    UpdateSpans,
}

/// Standard return shape from a data provider
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ProviderResultData>,
    pub loading: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderResultData {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Result of resolving one $reference
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HydrationResult {
    pub path: String,
    pub resolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Error from YAML parsing
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParseError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<f64>,
}

/// Error from validation — includes path to the problematic field
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationError {
    pub message: String,
    pub path: String,
}

/// Key-value map of component props
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Props {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, serde_json::Value>,
}

/// CSS style overrides
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StyleBlock {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Global style tokens defined in the YAML style section
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StyleTokens {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Maps data names to provider names
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataDeclarations {
    #[serde(flatten)]
    pub additional_properties: std::collections::HashMap<String, serde_json::Value>,
}
