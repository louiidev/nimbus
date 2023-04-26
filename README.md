# ☁️ nimbus ☁️
A small toy game engine built in rust, built to learn game engine &amp; graphics programming.


## About
* Plan is to make something similar to bevy but without having to use ECS
* Why another engine? I've always wanted to build a game engine and learn about how they work


## Goals
* 2D engine
* built in editor for managing/viewing entities on screen
* Fast & ergonomic, I want it to feel fun to use and to feel frictionless 
* Potentially 3D support, stretch goals


### Foundations
* winit OR sdl2 - For windowing, mouse & keyboard input
* gilrs - For gamepad support when using winit
* wgpu - For rendering backend, allowing the engine to support Vulkan, DX12 & Metal
* rodio - For audio playback and decoding
* arena(https://github.com/ChevyRay/arena) - For texture, font, audio storage and other contingious data that relys on id's for refs
