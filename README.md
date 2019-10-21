# wasm-marker-clusterer
A WebAssembly alternative to the popular MarkerClustererPlus library for Google Maps, compiled from Rust.

[![Build Status](https://travis-ci.org/stefan2718/webassembly-marker-clusterer.svg?branch=master)](https://travis-ci.org/stefan2718/webassembly-marker-clusterer)

## Motivation
With [Google Maps](https://developers.google.com/maps/documentation/javascript/marker-clustering), I've extensively used the [MarkerClustererPlus](https://github.com/googlemaps/v3-utility-library/tree/master/markerclustererplus) library for clustering map points together for a large real-estate company's web app. It worked mostly well-enough, however, when we zoomed out enough and started returning 10,000 cluster points (the maximum from Elastisearch), the performance really suffered, even on a high-end MacBook Pro. Whether this was the best way to handle this scenario is up for debate, but it got me wondering if it would be a good time to try out [WebAssembly](https://developer.mozilla.org/en-US/docs/WebAssembly).

Since clustering points together is a lot of math, it seems like something that could be sped up by running off of the main Javascript event loop in a compiled Wasm script. This should *hopefully* have the benefit of faster clustering, as well as not blocking rendering, allowing users to still interact with the site and map while clustering is happening in the background.
