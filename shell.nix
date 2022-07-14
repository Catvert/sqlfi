with import <nixpkgs> { };

stdenv.mkDerivation rec {
  name = "sqlife-env";
  buildInputs = [
    libxkbcommon
    libGL

    # WINIT_UNIX_BACKEND=wayland
    wayland

    # WINIT_UNIX_BACKEND=x11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    xorg.libX11
  ];

  LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
}
