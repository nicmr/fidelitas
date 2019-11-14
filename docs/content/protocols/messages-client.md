# Messages Client -> Server

## Play

#### Fields

- track_id : u64

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



