# Scenable

Scenable is a custom scenery manager for the [X-Plane 11](https://www.x-plane.com/) flight simulator. 


## Why Scenable?

Currently X-Plane appears to waste a lot of memory loading scenery and scenery library assets which it doesn't actually use. I measured a difference of about 10GB in memory usage when flying in an area with no custom scenery, but with lots of custom scenery in other areas currently enabled. Having lots of custom scenery enabled also makes the simulator slower to start. 

The purpose of this tool is to make the management of enabling/disabling custom scenery based on the area that you want to fly in as painless as possible. When disabling scenery with this tool, it will detect whether scenery libraries are currently un-used and will disable them automatically, and the same thing in reverse, when enabling an airport that uses a disabled library, it will also enable the library as required.
