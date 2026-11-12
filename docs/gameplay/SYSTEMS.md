# Gameplay Systems Design

GTASARUST integrates all gameplay systems through shared world/simulation state:

- **Player <-> Vehicles**: Entering a vehicle transitions player control from `OnFoot` to `InVehicle`, binds input to `VehicleHandlingState`, turns on vehicle engine/radio, and mounts the character to the seat transform.
- **Vehicles <-> Physics & Damage**: Each vehicle uses `handling.cfg` mass, drag, center of mass, suspension, drive bias, traction, and braking curves; collisions apply impulse damage and visual/mechanical degradation (engine smoke -> fire -> explosion).
- **Weapons <-> Entities & AI**: Weapon fire queries physics raycasts / projectiles, damages peds/vehicles, triggers panic/combat AI reactions, and notifies police crime detection.
- **Police & Wanted Level**: Witnessed crimes or proximity offenses raise wanted stars (`1`–`6`), escalating dispatch from patrol officers/cars to SWAT, FBI Ranchers, and Rhino tanks/Hydras.
- **Economy, Shops & Saves**: Money interacts with Ammu-Nation, Binco/SubUrban/ProLaps, Cluckin' Bell/Burger Shot, Pay 'n' Spray, and safehouse purchases, all persisted in versioned checksummed save files.
