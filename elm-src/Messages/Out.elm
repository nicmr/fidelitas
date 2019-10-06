module Messages.Out exposing (MsgKind(..), encodeMsg, compactJson)


import Json.Encode

type MsgKind = VolumeChange Int | Play Int | Pause | Stop | Resume

compactJson : MsgKind -> String
compactJson msg = encodeMsg msg |> Json.Encode.encode 0

encodeMsg : MsgKind -> Json.Encode.Value
encodeMsg msg =
    let
        typestring = ("type",  Json.Encode.string <| toString msg)
    in
        case msg of
            VolumeChange volume -> Json.Encode.object
                [ typestring
                , ("volume", Json.Encode.int volume)
                ]
            Play trackid -> Json.Encode.object
                [ typestring
                , ("track_id", Json.Encode.int trackid)
                ]1
            Pause -> Json.Encode.object [typestring ]
            Stop -> Json.Encode.object [typestring ]
            Resume -> Json.Encode.object [typestring ]

toString :  MsgKind -> String
toString msg = 
    case msg of
        VolumeChange _ -> "VolumeChange"
        Play _ -> "Play"
        Pause -> "Pause"
        Stop -> "Stop"
        Resume -> "Resume"