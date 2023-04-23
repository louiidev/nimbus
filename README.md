# ☁️ nimbus ☁️
A small toy game engine built in rust, built to learn game engine &amp; graphics programming.



## About
* Why another engine? I've always wanted to build a game engine and learn about how they work
* Who should use it? probably just me
* What's with the name? Hey, naming things is hard, I got this name from dragon ball and the flying nimbus!
* Heavily inspired by bevy and constantly used a reference when building features


## Goals
* 2D engine
* built in editor for managing/viewing entities on screen
* Fast & ergonomic, I want it to feel fun to use and to feel frictionless 
* Potentially 3D support, stretch goals


### Foundations
* sdl2 - For windowing, mouse, keyboard input & gamepad support
* wgpu - For rendering backend, allowing the engine to support Vulkan, DX12 & Metal
* rodio - For audio playback and decoding
* arena(https://github.com/ChevyRay/arena) - For texture, font, audio storage and other contingious data that relys on id's for refs
