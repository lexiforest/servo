/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// http://dev.w3.org/csswg/cssom-view/#the-screen-interface
[Exposed=Window]
interface Screen {
  readonly attribute double availWidth;
  readonly attribute double availHeight;
  readonly attribute long availLeft;
  readonly attribute long availTop;
  readonly attribute double width;
  readonly attribute double height;
  readonly attribute long left;
  readonly attribute long top;
  readonly attribute unsigned long colorDepth;
  readonly attribute unsigned long pixelDepth;
  readonly attribute boolean isExtended;
  readonly attribute ScreenOrientation orientation;
};
