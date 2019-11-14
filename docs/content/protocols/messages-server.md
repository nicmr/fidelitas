    FsChange,
    FsState{media: HashMap<u64, String>},
    RegisterSuccess,
    Error,
    VolumeChange{volume: u64}

# Messages Server -> Client

## Play

#### Fields

None
<!-- Should be: - track_id : u64 -->

#### Example

```json
{
    "type" : "Play",
    "track_id" : 14
}
```


## VolumeChange

#### Fields

- volume : u64

#### Example

```json
{
    "type" : "VolumeChange",
    "volume" : 84
}
```


## Pause

#### Fields

None

#### Example

```json
{
    "type" : "Stop"
}
```

## Stop

#### Fields

None

#### Example

```json
{
    "type" : "Stop"
}
```

## Resume

#### Fields

None

#### Example

```json
{
    "type" : "Resume"
}
```


## FsState

#### Fields


#### Example
```json
{
    "type" : FsState,
    "media" : {
        "1" : "title1",
        "2" : "title2"
    }
}
```

FsState{media: HashMap<u64, String>},


