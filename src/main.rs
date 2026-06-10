use ice_spar::api::instance_is::InstanceIS;

fn main() {
	let mut instance = InstanceIS::get_instance();
	let si = 0;
	instance.the_scene[si].add_model(
		"src/blueberry.fbx",
		&instance.app.device,
		&instance.app.queue,
	);
	instance.the_scene[si].build_pipeline(&instance.app.device, &instance.app.surface_conf);
	instance.run(si);
}
