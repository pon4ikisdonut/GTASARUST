use gtasarust_asset_pipeline::inspect_installation;
use std::path::Path;

#[test]
fn real_gtasa_installation_inspects_cleanly_when_present() {
    let dir = Path::new(r"E:\gtasa");
    if !dir.is_dir() {
        return;
    }
    let report = inspect_installation(dir).expect("E:\\gtasa inspection should succeed");
    assert!(report.validation.valid);
    assert!(report.manifest.models > 1000);
    assert!(report.manifest.textures > 1000);
    assert!(report.manifest.ide_objects > 1000);
    assert!(report.manifest.ipl_car_spawns > 500);
    assert!(report.manifest.handling_entries > 100);
    assert!(report.manifest.gxt_entries > 1000);
}
