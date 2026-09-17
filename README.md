# TechNoir

A lightweight, GPU-accelerated live wallpaper engine for Windows written in Rust.

## Overview

TechNoir creates interactive and procedural wallpapers behind desktop icons using 'wgpu' and native Windows desktop integration.

## Code Structure

- 'app': Application's entry point.
- 'engine': Core runtime that manages desktop hooks, display targets, GPU context, and wallpaper lifecycle.
- 'ui': System tray and management interface.
- 'wallpapers': Plugins for creating custom shader and procedural wallpapers.

## Status

Early development and initial overhaul.
