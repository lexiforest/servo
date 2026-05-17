Bimp Servo Patch Notes
======================

This directory is a local copy of Servo at commit
`c3d106b82e05af3d36db888e3085b9343320d511`.

Bimp carries this local copy so runtime fingerprint, transport, and automation
behavior can be patched repeatably while the upstream Servo dependency remains
explicit.

Current patch areas:

- JS-visible Chrome persona values.
- WebGL vendor and renderer values.
- WebDriver helper visibility.
- Automation input trust behavior is intentionally left to Servo's default event
  plumbing until Bimp has a real input dispatch path.
