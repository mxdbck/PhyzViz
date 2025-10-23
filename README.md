PhyzViz : library of small apps to vizualize simulations of different mathematical models of physical systems (such as those seen in this graduate course : https://uclouvain.be/en-cours-2025-linma2370). 

### Goals
 - [x] Simple pendulum numerically integrated bevy example
 - [x] Get wasm compilation working
 - [x] Double pendulum
 - [x] Add ribbon effect
 - [ ] Add performance metrics to quantify the impact of bloom and ribbon effects on framerate (and the terrible rk4 implementation)
 - [ ] Implement mesh ribbon instead of particle ribbon for better performance
 - [ ] Get Latex rendering working to show what systems of first order differential equations are being simulated on each app.
 - [ ] Add graphs showing position in phase space and other interesting metrics.
 - [ ] Add more examples.
 - [ ] Optimize 
 - [ ] Symplectic integrators.

### Notes
- Currently only RK4 is implemented (very poorly).