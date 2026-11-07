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
