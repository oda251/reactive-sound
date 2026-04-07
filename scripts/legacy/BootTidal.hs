:set -fno-warn-orphans -Wno-type-defaults -XMultiParamTypeClasses -XOverloadedStrings
:set prompt ""

import Sound.Tidal.Boot

default (Rational, Integer, Double, Pattern String)

-- Connect to SuperDirt via localhost (WSL2 mirrored networking)
tidalInst <- mkTidalWith [(superdirtTarget { oAddress = "127.0.0.1", oPort = 57120, oLatency = 0.05 }, [superdirtShape])] (defaultConfig {cFrameTimespan = 1/50, cProcessAhead = 1/20})

instance Tidally where tidal = tidalInst

:set prompt "tidal> "
:set prompt-cont ""
