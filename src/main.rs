//Internal Crates
mod graph;
use graph::Graph;
use graph::component_functions::*;
use graph::visualization_support::{show_aggregation,visualize_graph};

fn main() {
    let graph = Graph::from_csvs("citation_network\\edges.csv","citation_network\\nodes.csv").unwrap();

    graph.connected_components();
    
    let (component,num_components) = graph.connected_components();
    let component_sizes = count_components(&component, num_components);
    let component_scale = get_component_scale(&component, num_components,true);

    println!("Component Sizes: {:?}",component_sizes);
    println!("Component Scale: {:?}",component_scale);
    //Plot the component effectiveness
    show_aggregation(&component_scale, "plots\\component_aggregation.png").expect("Error in Aggregate Image Creation");
    //REALLY cool custom visual that shows the connectivity of the graph components
    visualize_graph("plots\\connected_components.png", &graph.outedges, &component,num_components,3.5).expect("Error in Connected Image Creation");
}




