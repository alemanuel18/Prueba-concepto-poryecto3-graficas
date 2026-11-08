# Contexto de trabajo — Proyecto 3: Space Travel

## Objetivo y restricciones
- Simulación explorable en Rust de un sistema solar inspirado en Super Mario Galaxy. El renderizador 3D es de software: transforma y rasteriza triángulos, calcula color por fragmento y usa z-buffer. Raylib se limita a ventana, entrada, framebuffer y HUD 2D.
- Mantener al menos un sol y varios planetas alineados con el plano eclíptico, con órbita y rotación dependientes del tiempo. La nave puede acercarse, rodearlos y alejarse en una escena continua.
- Código modular, repositorio ordenado, README con recursos, controles, créditos, instrucciones y video. El usuario pidió commits por sistema o cambio significativo.
- Es una demostración gráfica, sin vidas, daño, misiones ni necesidad de animar a Luigi. No copiar mecánicas de los videos de referencia.

## Recursos y preferencias del usuario
- Usar únicamente las mallas originales de los planetas, sin esferas añadidas. El sol sí es procedural.
- La nave de vuelo es Starship Mario: modelo grande `MarioFaceShipPlanet` y mini `MiniPlayerRocket`. La cubierta de la nave también es explorable.
- Los planetas tienen tamaños fijos (radios visuales de 10–13, órbitas de 55–298); solo la perspectiva cambia su tamaño aparente. Fluffy Bluff, Fluffy mini y Fortress mini requieren ignorar el `Z_UP` incorrecto de sus DAE.
- En vuelo siempre aparece `GalaxySkyUniverse`; los fondos específicos solo se componen al explorar el planeta correspondiente.
- Hay seis MP4 de gameplay en la raíz. Se examinaron fotogramas repartidos por toda su duración para guiar entradas, superficies, color y ambiente. El usuario pidió que esos videos no se incluyan en commits: `/*.mp4` los ignora. `media/galaxy-tour.mp4` es el video generado por el proyecto y está versionado.
- Yoshi y Throwback muestran caminos y terrazas; Freezeflame, superficies de hielo entre paredes; Fluffy Bluff, jardines y praderas; el volcán, roca junto a lava; Starship Mario, cubierta y partes curvas de la cabeza. Los modelos disponibles no incluyen todas las plataformas del video de Blistering Core.

## Arquitectura actual
- `src/renderer.rs`: cámara, transformaciones, rasterización CPU, z-buffer, sombreado y fondos.
- `src/asset.rs`: formato compacto de mallas y sol procedural. `tools/convert_assets.py` transforma DAE/PNG y el OBJ/MTL/PNG de Luigi a `assets/processed/`. Los ZIP originales se ignoran en Git. El OBJ conserva los colores de Luigi mejor que su DAE.
- `src/scene.rs`: cinco planetas, órbitas, entradas locales elegidas sobre sus modelos, superficies transitables, distribución decorativa y colisiones con paredes. Los objetos se distribuyen sobre triángulos aptos para caminar.
- `src/controls.rs`: vuelo, viajes rápidos, anillo, caminar y colisiones con decoración sólida. `1`–`5` viaja cerca de los planetas; `E` usa el anillo; `F` explora Starship; `R` regresa al vuelo.
- `src/avatar.rs`: Luigi estático, altura mundial 0.8, radio de apoyo 0.14, orientación según la normal del piso y cámara de seguimiento que se acerca cuando la malla obstruye la vista. El recurso tiene los brazos extendidos.
- `src/main.rs`: carga de recursos, composición de escena, HUD, capturas y video generado. Hay LOD, descarte y fondos por planeta.

## Caminabilidad y límites actuales
- `closest_floor` exige triángulos cercanos, pendiente transitable (normal · vertical local ≥ 0.78), continuidad de altura y normal. `walk_step` divide cada desplazamiento, exige apoyo de los pies y detiene el torso ante paredes. En controles también se bloquean objetos decorativos sólidos.
- Yoshi, Fluffy, Freezeflame Ice y Throwback usan vertical global +Y. Volcano Sandwich y Starship Mario usan vertical radial local. La lava roja no cuenta como piso ni como zona para colocar objetos.
- Entradas actuales: pradera central de Yoshi, jardín de Fluffy, base nevada de Freezeflame, roca de Volcano, área verde de Throwback y cubierta de Starship. Son aproximaciones sobre las mallas suministradas, no reproducciones completas de los niveles de los videos.
- Hay siete pruebas que cubren escala fija, seguimiento del piso, paredes, gravedad radial, lava y recorrido con el radio del personaje: al menos 12 pasos seguidos desde cada entrada planetaria y 30 sobre la nave. `cargo test --offline` pasó tras el cambio de Luigi.

## Estado de entrega
- `README.md` documenta controles, módulos, recursos y límites. `media/galaxy-tour.mp4` dura 29 segundos y muestra vuelo, planetas, Luigi y cubierta de la nave.
- El trabajo está en commits separados para importación de Luigi (`c9ea334`), navegación/cámara/colisiones (`3b93644`) y documentación/video generado (`80ef950`). No hay remoto Git ni nombres de integrantes confirmados; no inventarlos.
- Para cambios futuros, actualizar este archivo y el README con el comportamiento real; regenerar el video de demostración si cambia la escena. Mantener los seis MP4 de referencia fuera de Git.
