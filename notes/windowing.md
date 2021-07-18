# Windowing

`main()` is responsible for creating the window context and setting up the initial systems for the game.

Currently, it does the following:

- sdl2
    - inits a video subsystem which sets:
        - OpenGL Profile Core 3.3
- sdl2 ttf_context
- creates the actual window
- creates a canvas from the window, binding a supported opengl driver
- creates a texture creator from the canvas
- creates an event pump from the sdl context
- creates the following systems: 
    - SpriteManager
    - FontManager
    - Game struct -- (Should be removed)
    - Specs World struct
    - InputSystem
    - Physics(System)
    - Renderer
- registers the world struct with our components
- creates a few entities
- runs the game loop

# Game loop
<div>
https://gafferongames.com/post/fix_your_timestep/<br>
https://dewitters.com/dewitters-gameloop/
</div>

We want the game loop to run as follows:

- receive player input
- render the underlying game systems at approximately 30fps
- render the game at 60fps

This part was implemented from my understanding of the above articles.




