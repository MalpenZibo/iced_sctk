# Iced SCTK

This repository is a layer between the iced framework and the wayland protocol (layer surface, session lock, etc).
I primary use this repository for my [ashell](https://github.com/MalpenZibo/ashell) project.

The code in this repository is a port of the work done by the `pop_os` team, 
who created a fork of iced that supports some specific Wayland protocols: https://github.com/pop-os/iced.

I tried to use only the pieces necessary to support Wayland without having to use 
all the changes applied by their fork on the original iced repository.

In this way, I am able to use the original version of iced to create wayland layer shells.

Currently, some functionalities of the `pop_os` iced version are not working, 
and the code in this repository is a simple copy-paste with some fixes, so I cannot guarantee its functionality.

I recommend using the original `pop_os` fork or following other projects with the same goal, 
such as https://github.com/waycrate/exwlshelleventloop.


Iced Pop-os reference commit dd2e93a54df9e5a711833d9551532e1794eda60f
