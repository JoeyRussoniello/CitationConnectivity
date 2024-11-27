//Import all graph functions
mod graph;
use graph::Graph;
use graph::component_functions::*;
use graph::visualization_support::show_aggregation;
fn main() {
    let graph = Graph::from_csvs("citation_network\\edges.csv","citation_network\\nodes.csv").unwrap();
    
    let (component,num_components) = graph.connected_components();
    let component_sizes = count_components(&component, num_components);
    let component_scale = get_component_scale(&component, num_components,true);

    println!("Component Sizes: {:?}",component_sizes);
    println!("Component Scale: {:?}",component_scale);
    //Plot the component effectiveness
    show_aggregation(&component_scale, "plots\\component_aggregation.png").expect("Error in Aggregate Image Creation");
    //REALLY cool custom visual that shows the connectivity of the graph components
    graph.visualize_connectivity("plots\\connected_components.png", 3.0, (1024,1024),"All Research Connected Components").unwrap();

    let subgraphs = graph.calculate_subgraphs();
    //For each research subject, calculate statistics about its component and create a visualization
    for (subject, subgraph) in subgraphs.iter(){
        let (component, num_components) = subgraph.connected_components();
        let component_scale = get_component_scale(&component, num_components, true);
        println!("Papers in {} have {} components. {:.2} of the data is captured in with the following component scale::\n{:?}\n",subject,num_components,component_scale[1],component_scale);
        // subgraph.visualize_connectivity(
        //     &format!("plots\\subgraphs\\{}_connectivity.png",subject),
        //     3.0, (1024,1024), 
        //     &format!("Connectivity of Research Papers in {}",subject)).unwrap();
    }
}
