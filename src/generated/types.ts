
/**
 * @type { Direction }
 * @description Layout direction for a region
 */
export type Direction =
  | 'vertical'
  | 'horizontal'
  | 'grid'
;

/**
 * @type { Justify }
 * @description Content justification within a region
 */
export type Justify =
  | 'start'
  | 'center'
  | 'end'
  | 'space-between'
;

/**
 * @type { EventType }
 * @description Standard event types mapped to library-specific prop names by the adapter
 */
export type EventType =
  | 'click'
  | 'input'
  | 'change'
  | 'submit'
  | 'focus'
  | 'blur'
;

/**
 * @type { ActionType }
 * @description Built-in action types the engine can execute
 */
export type ActionType =
  | 'show'
  | 'hide'
  | 'toggle'
  | 'navigate'
  | 'back'
  | 'notify'
  | 'scroll-to'
  | 'focus'
  | 'enable'
  | 'disable'
  | 'update'
  | 'reset'
  | 'fetch'
  | 'refresh'
  | 'update-region'
  | 'update-props'
  | 'multiple'
  | 'handler'
;

/**
 * @type { PageTree }
 * @description Top-level parsed YAML template
 */
export type PageTree = {
  /**
   * @type { string }
   * @memberof PageTree
   */
  page: string;
  /**
   * @type { PageTreeRegions }
   * @memberof PageTree
   */
  regions: PageTreeRegions;
  /**
   * @type { StyleTokens }
   * @memberof PageTree
   */
  style: StyleTokens | null;
  /**
   * @type { DataDeclarations }
   * @memberof PageTree
   */
  data: DataDeclarations | null;
  /**
   * @type { Interaction[] }
   * @memberof PageTree
   */
  interactions: Interaction[] | null;
};
/**
 * @type { PageTreeRegions }
 */
export type PageTreeRegions = {
  [keys: string]: Region;
};

/**
 * @type { Region }
 * @description A named layout container that holds components or sub-regions
 */
export type Region = {
  /**
   * @type { string }
   * @memberof Region
   */
  id: string | null;
  /**
   * @type { Direction }
   * @memberof Region
   */
  direction: Direction | null;
  /**
   * @type { number }
   * @memberof Region
   */
  gap: number | null;
  /**
   * @type { Justify }
   * @memberof Region
   */
  justify: Justify | null;
  /**
   * @type { number }
   * @memberof Region
   */
  span: number | null;
  /**
   * @type { number }
   * @memberof Region
   */
  columns: number | null;
  /**
   * @type { number }
   * @memberof Region
   */
  layer: number | null;
  /**
   * @type { number }
   * @memberof Region
   */
  'collapse-below': number | null;
  /**
   * @type { number }
   * @memberof Region
   */
  'stack-below': number | null;
  /**
   * @type { RegionRegions }
   * @memberof Region
   */
  regions: RegionRegions | null;
  /**
   * @type { Component[] }
   * @memberof Region
   */
  components: Component[] | null;
  /**
   * @type { StyleBlock }
   * @memberof Region
   */
  style: StyleBlock | null;
};
/**
 * @type { RegionRegions }
 */
export type RegionRegions = {
  [keys: string]: Region;
};

/**
 * @type { Component }
 * @description A renderable UI component with type, props, and optional layout/style
 */
export type Component = {
  /**
   * @type { string }
   * @memberof Component
   */
  type: string;
  /**
   * @type { string }
   * @memberof Component
   */
  id: string | null;
  /**
   * @type { Props }
   * @memberof Component
   */
  props: Props | null;
  /**
   * @type { string }
   * @memberof Component
   */
  data: string | null;
  /**
   * @type { number }
   * @memberof Component
   */
  span: number | null;
  /**
   * @type { number }
   * @memberof Component
   */
  colSpan: number | null;
  /**
   * @type { number }
   * @memberof Component
   */
  rowSpan: number | null;
  /**
   * @type { StyleBlock }
   * @memberof Component
   */
  style: StyleBlock | null;
};

/**
 * @type { Interaction }
 * @description Maps a component event to a built-in action or handler
 */
export type Interaction = {
  /**
   * @type { string }
   * @memberof Interaction
   */
  on: string;
  /**
   * @type { string }
   * @memberof Interaction
   */
  do: string | null;
  /**
   * @type { string }
   * @memberof Interaction
   */
  target: string | null;
  /**
   * @type { string }
   * @memberof Interaction
   */
  handler: string | null;
  /**
   * @type { string }
   * @memberof Interaction
   */
  to: string | null;
  /**
   * @type { string }
   * @memberof Interaction
   */
  message: string | null;
};

/**
 * @type { Action }
 * @description An action the engine executes (built-in or returned by handler)
 */
export type Action = {
  /**
   * @type { ActionType }
   * @memberof Action
   */
  type: ActionType;
  /**
   * @type { string }
   * @memberof Action
   */
  target: string | null;
  /**
   * @type { string }
   * @memberof Action
   */
  to: string | null;
  /**
   * @type { string }
   * @memberof Action
   */
  message: string | null;
  /**
   * @type { string }
   * @memberof Action
   */
  variant: string | null;
  /**
   * @type { Props }
   * @memberof Action
   */
  props: Props | null;
  /**
   * @type { Component[] }
   * @memberof Action
   */
  components: Component[] | null;
  /**
   * @type { Action[] }
   * @memberof Action
   */
  actions: Action[] | null;
  /**
   * @type { ActionData }
   * @memberof Action
   */
  data: ActionData | null;
};
/**
 * @type { ActionData }
 */
export type ActionData = {
  [keys: string]: unknown;
};

/**
 * @type { LayoutNode }
 * @description Layout-only view of a region — no component props
 */
export type LayoutNode = {
  /**
   * @type { string }
   * @memberof LayoutNode
   */
  name: string;
  /**
   * @type { string }
   * @memberof LayoutNode
   */
  id: string | null;
  /**
   * @type { Direction }
   * @memberof LayoutNode
   */
  direction: Direction;
  /**
   * @type { number }
   * @memberof LayoutNode
   */
  gap: number;
  /**
   * @type { Justify }
   * @memberof LayoutNode
   */
  justify: Justify;
  /**
   * @type { number }
   * @memberof LayoutNode
   */
  span: number;
  /**
   * @type { number }
   * @memberof LayoutNode
   */
  columns: number | null;
  /**
   * @type { number }
   * @memberof LayoutNode
   */
  layer: number;
  /**
   * @type { number }
   * @memberof LayoutNode
   */
  collapseBelow: number | null;
  /**
   * @type { number }
   * @memberof LayoutNode
   */
  stackBelow: number | null;
  /**
   * @type { StyleBlock }
   * @memberof LayoutNode
   */
  style: StyleBlock | null;
  /**
   * @type { LayoutNode[] }
   * @memberof LayoutNode
   */
  childRegions: LayoutNode[] | null;
  /**
   * @type { ComponentSlotRef[] }
   * @memberof LayoutNode
   */
  childSlots: ComponentSlotRef[] | null;
};

/**
 * @type { ComponentSlotRef }
 * @description Reference to a component in the flat component list
 */
export type ComponentSlotRef = {
  /**
   * @type { number }
   * @memberof ComponentSlotRef
   */
  index: number;
  /**
   * @type { number }
   * @memberof ComponentSlotRef
   */
  span: number;
  /**
   * @type { number }
   * @memberof ComponentSlotRef
   */
  colSpan: number | null;
  /**
   * @type { number }
   * @memberof ComponentSlotRef
   */
  rowSpan: number | null;
};

/**
 * @type { ComponentEntry }
 * @description A component extracted from the tree with its region path
 */
export type ComponentEntry = {
  /**
   * @type { Component }
   * @memberof ComponentEntry
   */
  component: Component;
  /**
   * @type { string }
   * @memberof ComponentEntry
   */
  regionPath: string;
};

/**
 * @type { SplitResult }
 * @description Result of splitting a PageTree
 */
export type SplitResult = {
  /**
   * @type { LayoutNode[] }
   * @memberof SplitResult
   */
  layout: LayoutNode[];
  /**
   * @type { ComponentEntry[] }
   * @memberof SplitResult
   */
  components: ComponentEntry[];
  /**
   * @type { StyleTokens }
   * @memberof SplitResult
   */
  styles: StyleTokens | null;
  /**
   * @type { SplitResultData }
   * @memberof SplitResult
   */
  data: SplitResultData | null;
};
/**
 * @type { SplitResultData }
 */
export type SplitResultData = {
  [keys: string]: unknown;
};

/**
 * @type { ContainerNode }
 * @description A layout container with CSS properties — output of the arranger
 */
export type ContainerNode = {
  /**
   * @type { string }
   * @memberof ContainerNode
   */
  id: string;
  /**
   * @type { LayoutTypeEnum }
   * @memberof ContainerNode
   */
  layoutType: LayoutTypeEnum;
  /**
   * @type { DirectionEnum }
   * @memberof ContainerNode
   */
  direction: DirectionEnum;
  /**
   * @type { number }
   * @memberof ContainerNode
   */
  gap: number | null;
  /**
   * @type { Justify }
   * @memberof ContainerNode
   */
  justify: Justify | null;
  /**
   * @type { number }
   * @memberof ContainerNode
   */
  layer: number | null;
  /**
   * @type { number }
   * @memberof ContainerNode
   */
  columns: number | null;
  /**
   * @type { number }
   * @memberof ContainerNode
   */
  collapseBelow: number | null;
  /**
   * @type { number }
   * @memberof ContainerNode
   */
  stackBelow: number | null;
  /**
   * @type { boolean }
   * @memberof ContainerNode
   */
  hidden: boolean | null;
  /**
   * @type { StyleBlock }
   * @memberof ContainerNode
   */
  style: StyleBlock | null;
  /**
   * @type { ContainerChild[] }
   * @memberof ContainerNode
   */
  children: ContainerChild[] | null;
};
/**
 * @type { LayoutTypeEnum }
 */
export type LayoutTypeEnum =
  | 'flex'
  | 'grid'
;
/**
 * @type { DirectionEnum }
 */
export type DirectionEnum =
  | 'row'
  | 'column'
;

/**
 * @type { ContainerChild }
 * @description Either a nested container or a component slot
 */
export type ContainerChild = {
  /**
   * @type { ContainerNode }
   * @memberof ContainerChild
   */
  container: ContainerNode | null;
  /**
   * @type { ComponentSlot }
   * @memberof ContainerChild
   */
  slot: ComponentSlot | null;
};

/**
 * @type { ComponentSlot }
 * @description A placeholder in the container tree for a component
 */
export type ComponentSlot = {
  /**
   * @type { string }
   * @memberof ComponentSlot
   */
  componentId: string;
  /**
   * @type { number }
   * @memberof ComponentSlot
   */
  span: number | null;
  /**
   * @type { number }
   * @memberof ComponentSlot
   */
  colSpan: number | null;
  /**
   * @type { number }
   * @memberof ComponentSlot
   */
  rowSpan: number | null;
};

/**
 * @type { ContainerUpdate }
 * @description A diff describing a layout change from resize recalculation
 */
export type ContainerUpdate = {
  /**
   * @type { TypeEnum }
   * @memberof ContainerUpdate
   */
  type: TypeEnum;
  /**
   * @type { string }
   * @memberof ContainerUpdate
   */
  containerId: string;
  /**
   * @type { string }
   * @memberof ContainerUpdate
   */
  newDirection: string | null;
};
/**
 * @type { TypeEnum }
 */
export type TypeEnum =
  | 'hide'
  | 'show'
  | 'change-direction'
  | 'update-spans'
;

/**
 * @type { ProviderResult }
 * @description Standard return shape from a data provider
 */
export type ProviderResult = {
  /**
   * @type { ProviderResultData }
   * @memberof ProviderResult
   */
  data: ProviderResultData | null;
  /**
   * @type { boolean }
   * @memberof ProviderResult
   */
  loading: boolean;
  /**
   * @type { string }
   * @memberof ProviderResult
   */
  error: string | null;
};
/**
 * @type { ProviderResultData }
 */
export type ProviderResultData = {
  [keys: string]: unknown;
};

/**
 * @type { HydrationResult }
 * @description Result of resolving one $reference
 */
export type HydrationResult = {
  /**
   * @type { string }
   * @memberof HydrationResult
   */
  path: string;
  /**
   * @type { boolean }
   * @memberof HydrationResult
   */
  resolved: boolean;
  /**
   * @type { string }
   * @memberof HydrationResult
   */
  value: string | null;
};

/**
 * @type { ParseError }
 * @description Error from YAML parsing
 */
export type ParseError = {
  /**
   * @type { string }
   * @memberof ParseError
   */
  message: string;
  /**
   * @type { number }
   * @memberof ParseError
   */
  line: number | null;
  /**
   * @type { number }
   * @memberof ParseError
   */
  column: number | null;
};

/**
 * @type { ValidationError }
 * @description Error from validation — includes path to the problematic field
 */
export type ValidationError = {
  /**
   * @type { string }
   * @memberof ValidationError
   */
  message: string;
  /**
   * @type { string }
   * @memberof ValidationError
   */
  path: string;
};

/**
 * @type { Props }
 * @description Key-value map of component props
 */
export type Props = {
  [keys: string]: unknown;
};

/**
 * @type { StyleBlock }
 * @description CSS style overrides
 */
export type StyleBlock = {
  [keys: string]: unknown;
};

/**
 * @type { StyleTokens }
 * @description Global style tokens defined in the YAML style section
 */
export type StyleTokens = {
  [keys: string]: unknown;
};

/**
 * @type { DataDeclarations }
 * @description Maps data names to provider names
 */
export type DataDeclarations = {
  [keys: string]: unknown;
};

