/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

dictionary NavigatorUABrandVersion {
  required DOMString brand;
  required DOMString version;
};

dictionary UADataValues {
  required sequence<NavigatorUABrandVersion> brands;
  required sequence<NavigatorUABrandVersion> fullVersionList;
  required boolean mobile;
  required DOMString platform;
  required DOMString platformVersion;
  required DOMString architecture;
  required DOMString bitness;
  required DOMString model;
  required DOMString uaFullVersion;
  required DOMString fullVersion;
};

[Exposed=(Window,Worker)]
interface NavigatorUAData {
  readonly attribute /* FrozenArray<NavigatorUABrandVersion> */ any brands;
  readonly attribute boolean mobile;
  readonly attribute DOMString platform;
  Promise<UADataValues> getHighEntropyValues(sequence<DOMString> hints);
};

partial interface Navigator {
  [SameObject] readonly attribute NavigatorUAData userAgentData;
};

partial interface WorkerNavigator {
  [SameObject] readonly attribute NavigatorUAData userAgentData;
};
