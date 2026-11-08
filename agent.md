# Contexto de trabajo — Proyecto 3: Space Travel

## Objetivo
Crear en Rust una simulación explorable de un sistema solar inspirado en Super Mario Galaxy. La escena se dibuja mediante un renderer 3D de software propio sobre un framebuffer; raylib solo crea la ventana, recibe entradas y presenta el framebuffer/HUD 2D.

## Requisitos obligatorios del documento `PROYECTO_FINAL GRAFICAS 2026.docx`
- Sol, nave controlable y al menos un planeta que orbita y rota usando tiempo transcurrido.
- Nave que se acerca, rodea y se aleja sin pausa, carga ni cambio de escena; cámara de seguimiento orientable.
- Carga de modelos 3D y transformaciones modelo/vista/proyección, z-buffer y al menos un shader de fragmento propio.
- Código, recursos, instrucciones, controles y video de vuelo en README. Explicar contribuciones, recursos externos e IA.
- Raylib no puede dibujar modelos, usar su cámara 3D, materiales, shaders GPU ni física.

## Intención del usuario
- Cinco planetas y una nave con versiones pequeñas/low-poly en `assets/*.zip`, además de cielos, estructuras, plantas y personajes.
- Planetas con identidades visuales; lava, hielo, agua y metal deben lucir convincentes.
- Anillo estelar para llegar a la zona inicial de cada planeta; invocar otro para volver a la nave. Sin vidas, daño ni mecánicas de juego.
- README ordenado, video incrustado cuando exista y commits por sistema/planeta/cambio importante.

## Estado inicial
`Cargo.toml` incluye raylib 6. `src/main.rs` es Hello World. Los modelos están en ZIP principalmente como DAE y PNG. Hay PDF de apoyo en la raíz. No hay implementación previa.

## Criterios de implementación
Mantener órbitas y cámara en una escena continua. Proteger rendimiento con detalle por distancia y descarte visible. Evitar guardar archivos de compilación o recursos temporales. Documentar claramente lo implementado y las limitaciones reales. No afirmar la existencia de un video ni enlace remoto que no se haya creado.

## Implementación actual
Pipeline Rust modular en `src/`; assets DAE/PNG convertidos a `assets/processed/` mediante `tools/convert_assets.py`. Cinco planetas, sol procedural, nave original, anillos, modo superficie, fondos por planeta, shaders CPU, z-buffer, LOD, descarte y video `media/galaxy-tour.mp4`. Los archivos ZIP originales se ignoran en Git; las mallas/cielos compactos sí están versionados. `README.md` contiene controles y créditos. No hay remoto Git ni nombres de integrantes confirmados; no inventarlos.

## Correcciones del usuario incorporadas
No añadir esferas a los planetas: se usan solo las mallas DAE convertidas. La nave de vuelo es Starship Mario (`MarioFaceShipPlanet`) y su mini (`MiniPlayerRocket`) con cambio de cámara mediante `C`. Los planetas mantienen radios visuales constantes de 10–13 unidades y órbitas de 55–298; solo la perspectiva los hace crecer al acercarse. Fluffy Bluff, Fluffy mini y Fortress mini requieren ignorar `Z_UP` del DAE porque sus vértices ya están orientados en Y. El fondo espacial es siempre GalaxySkyUniverse; los cinco fondos adicionales se componen sobre estrellas únicamente en superficie planetaria, con proporción 1:1 y alfa. El sol sigue siendo procedural.

## Exploración de superficies
Los videos de gameplay compartidos por el usuario son referencias de zonas transitables, ambientación y color, no una petición de copiar mecánicas. Los enlaces de YouTube no pudieron reproducirse desde este entorno; no afirmar que se observaron. La implementación se apoya en la geometría y los recursos locales. `Mode::Surface` puede apuntar a un planeta o a Starship Mario; `F` permite explorar la cubierta de la nave y `R` regresa al vuelo. La búsqueda de piso (`closest_floor`) acepta solo triángulos cercanos con pendiente razonable, continuidad de altura y continuidad de normal. Cuatro planetas tienen vertical global +Y; Volcano Sandwich y Starship Mario tienen vertical radial local. El movimiento se bloquea en bordes y paredes. Los objetos decorativos se colocan solo en triángulos transitables. Una prueba verifica 30 pasos seguidos en los cinco planetas y la nave.

## Estado de entrega
`media/galaxy-tour.mp4` dura 29 segundos y termina con la cubierta de Starship Mario. `tools/convert_assets.py` recupera también materiales DAE asignados directamente a primitivas; se regeneraron las mallas afectadas. No hay remoto Git ni nombres de integrantes confirmados; no inventarlos. Mantener README y video sincronizados con los cambios posteriores y realizar commits por cambio importante.
