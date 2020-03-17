module Messages.In exposing (CurrentMedia, PlaybackState(..), IncomingMessage(..), MediaMeta, messageDecoder)

import Json.Decode exposing (field, succeed, fail, Decoder, string)
import Dict exposing (Dict)


type IncomingMessage = RegisterSuccess 
    | PlaybackChange PlaybackState
    | FsChange
    | PlayerState PlaybackState (Dict String MediaMeta)
    | Error
    | VolumeChange Int

type PlaybackState = Playing CurrentMedia | Paused CurrentMedia | Stopped

type alias MediaMeta =
  { length: Int
  , title: String
  , album: String
  , artist: String
  }

type alias CurrentMedia =
  { id: Int
  , lengthMillis: Int
  , progress: Int
  }

decodeMediaMeta : Decoder MediaMeta
decodeMediaMeta = 
  Json.Decode.map4 MediaMeta
    (field "length" Json.Decode.int )
    (field "title" Json.Decode.string )
    (field "album" Json.Decode.string )
    (field "artist" Json.Decode.string )

decodeCurrentMedia : Decoder CurrentMedia
decodeCurrentMedia =
  Json.Decode.map3 CurrentMedia
    (field "id" Json.Decode.int)
    (field "length" Json.Decode.int)
    (field "progress" Json.Decode.int)

decodePlaybackState : Decoder PlaybackState
decodePlaybackState =
  field "playback-type" Json.Decode.string
  |> Json.Decode.andThen (\playbackKind ->
    case playbackKind of
      "Playing" ->
        Json.Decode.map Playing
          ( field "current_media" decodeCurrentMedia)
      "Paused" ->
        Json.Decode.map Paused
          ( field "current_media" decodeCurrentMedia)
      "Stopped" ->
        succeed Stopped
      _ ->
        fail <| "Can't decode playbackState"
    )

messageDecoder : Decoder IncomingMessage
messageDecoder =
  field "type" Json.Decode.string
  |> Json.Decode.andThen (\kind ->
    case kind of
      "PlayerState" -> playerStateDecoder
      "VolumeChange" -> volumeChangeDecoder
      "PlaybackChange" -> playbackChangeDecoder
      "FsChange" -> succeed FsChange
      "RegisterSuccess" -> succeed RegisterSuccess
      "Error" -> succeed Error
      _ -> fail "Can't decode message kind"     
    )

playbackChangeDecoder : Decoder IncomingMessage
playbackChangeDecoder =
  Json.Decode.map PlaybackChange
    ( field "playback_state" decodePlaybackState)

playerStateDecoder : Decoder IncomingMessage
playerStateDecoder =
  Json.Decode.map2 PlayerState
    ( field "playback_state" decodePlaybackState)
    ( field "media" (Json.Decode.dict decodeMediaMeta))

volumeChangeDecoder : Decoder IncomingMessage
volumeChangeDecoder =
  Json.Decode.map VolumeChange
    (field "volume" Json.Decode.int)