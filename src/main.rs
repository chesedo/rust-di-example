use di::{DependencyContainer, DependencyContainerImpl};

fn main() {
    let dependency_container = DependencyContainerImpl::new(None);

    dependency_container.clone().worker().work();

    {
        println!("Clone should not make new singleton instances");
        dependency_container.clone().worker().work();
        dependency_container.clone().worker().work();
    }

    {
        println!("New scope should not make new scope instances");
        let d2 = dependency_container.new_scope();
        d2.worker().work();
        d2.worker().work();

        let d3 = dependency_container.new_scope();
        d3.clone().worker().work();
        d3.clone().worker().work();
    }
}
