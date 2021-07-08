# OxideXR (temporary name)

As OpenXR is relatively young in comparison to OpenVR its runtimes lack many features which VR users can take for granted. 
One of the most noticable missing features is input binding customisation.

OxideXR intends to be universal solution to this issue by implementing input binding customisation on top of the active OpenXR runtime. 
As this is achived through an implicit layer this should work with any OpenXR runtime with minimal compatibility issues.

In the future I hope to expand OxideXR to provide more features such as spec extensions which the runtime itself does not support.

## TODO

- [x] Working rust OpenXR layer
- [x] Interception and serialization of an applications Actionsets, Actions, and default bindings 
- [x] Binding customisation through json files
- [ ] Flat GUI
- [ ] Automated left handed binding generation
- [ ] VR GUI
- [ ] Support for XR_VALVE_analog_threshold
- [ ] Design an extension to allow coms between the layer and applications

## Possible features

- [ ] Implement XR_MSFT_controller_model
