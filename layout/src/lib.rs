use dioxus::core::ElementId;
use dioxus_native_core::real_dom::NodeType;
use freya_layers::{Layers, NodeArea, NodeData};
use freya_node_state::node::{DirectionMode, SizeMode};

fn calculate_area(node_data: &NodeData, mut area: NodeArea, parent_area: NodeArea) -> NodeArea {
    let calculate = |value: SizeMode, area_value: f32, parent_area_value: f32| -> f32 {
        match value {
            SizeMode::Manual(v) => v,
            SizeMode::Percentage(per) => (parent_area_value / 100.0 * per).round(),
            SizeMode::Auto => area_value,
        }
    };

    let calculate_min = |value: SizeMode, area_value: f32, parent_area_value: f32| -> f32 {
        match value {
            SizeMode::Manual(v) => {
                if v > area_value {
                    v
                } else {
                    area_value
                }
            }
            SizeMode::Percentage(per) => {
                let by_per = (parent_area_value / 100.0 * per).round();
                if by_per > area_value {
                    by_per
                } else {
                    area_value
                }
            }
            SizeMode::Auto => area_value,
        }
    };

    area.width = calculate(node_data.size.width, area.width, parent_area.width);
    area.height = calculate(node_data.size.height, area.height, parent_area.height);

    if SizeMode::Auto == node_data.size.height {
        if let NodeType::Element { tag, .. } = &node_data.node.node_type {
            if tag == "label" {
                area.height = 18.0;
            }
        }
    }

    area.height = calculate_min(node_data.size.min_height, area.height, parent_area.height);
    area.width = calculate_min(node_data.size.min_width, area.width, parent_area.width);

    area
}

pub fn calculate_node<T>(
    node_data: &NodeData,
    remaining_area: NodeArea,
    parent_area: NodeArea,
    resolver_options: &mut T,
    layers: &mut Layers,
    node_resolver: fn(&ElementId, &mut T) -> Option<NodeData>,
    inherited_relative_layer: i16,
) -> NodeArea {
    let mut node_area = calculate_area(node_data, remaining_area, parent_area);
    let mut is_text = false;

    // Returns a tuple, the first element is the layer in which the current node must be added
    // and the second indicates the layer that it's children must inherit
    let (node_layer, inherited_relative_layer) =
        layers.calculate_layer(node_data, inherited_relative_layer);

    let padding = node_data.size.padding;
    let horizontal_padding = padding.1 + padding.3;
    let vertical_padding = padding.0 + padding.2;

    // Area that is available consideing the parent area
    let mut remaining_inner_area = NodeArea {
        x: node_area.x + padding.3,
        y: node_area.y + padding.0,
        width: node_area.width - horizontal_padding,
        height: node_area.height - vertical_padding,
    };
    // Visible area occupied by the child elements
    let inner_area = remaining_inner_area.clone();

    remaining_inner_area.y += node_data.size.scroll_y;
    remaining_inner_area.x += node_data.size.scroll_x;

    match &node_data.node.node_type {
        NodeType::Element { children, .. } => {
            for child in children {
                let child_node = node_resolver(child, resolver_options);

                if let Some(child_node) = child_node {
                    let child_node_area = calculate_node::<T>(
                        &child_node,
                        remaining_inner_area,
                        inner_area,
                        resolver_options,
                        layers,
                        node_resolver,
                        inherited_relative_layer,
                    );

                    match node_data.size.direction {
                        DirectionMode::Vertical => {
                            remaining_inner_area.y = child_node_area.y + child_node_area.height;
                        }
                        DirectionMode::Horizontal => {
                            remaining_inner_area.x = child_node_area.x + child_node_area.width;
                        }
                        DirectionMode::Both => {
                            remaining_inner_area.y = child_node_area.y + child_node_area.height;
                            remaining_inner_area.x = child_node_area.x + child_node_area.width;
                        }
                    }

                    remaining_inner_area.height -= child_node_area.height;
                    remaining_inner_area.width -= child_node_area.width;

                    if child_node_area.width > remaining_inner_area.width
                        || remaining_inner_area.width == 0.0
                    {
                        remaining_inner_area.width = child_node_area.width;
                    }

                    if child_node_area.height > remaining_inner_area.height
                        || remaining_inner_area.height == 0.0
                    {
                        remaining_inner_area.height = child_node_area.height;
                    }
                }
            }
        }
        NodeType::Text { .. } => {
            is_text = true;
        }
        NodeType::Placeholder => {}
    }

    if !is_text {
        if let SizeMode::Auto = node_data.size.width {
            node_area.width = remaining_inner_area.x - node_area.x + padding.1;
        }

        if let SizeMode::Auto = node_data.size.height {
            node_area.height = remaining_inner_area.y - node_area.y + padding.0;
        }
    }

    // Registers the element in the Layers handler
    layers.add_element(node_data, &node_area, node_layer);

    node_area
}
