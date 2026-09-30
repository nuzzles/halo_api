// Fixture generator: copy into the grammar package at LevelUp d61443ef59268ad734355db8e9974f68db5ca6d0.
// Run only TestHaloRustReferenceContexts with HALO_FILM_CORPUS and HALO_REFERENCE_OUTPUT set.
// All expectations come from the pinned Go implementation, never Rust.
package grammar

import (
	"compress/zlib"
	"encoding/json"
	"levelup/go-api/internal/games/halo_infinite/film/internal/source"
	"levelup/go-api/internal/games/halo_infinite/film/types"
	"os"
	"path/filepath"
	"testing"
)

func TestHaloRustReferenceContexts(t *testing.T) {
	root := os.Getenv("HALO_FILM_CORPUS")
	output := os.Getenv("HALO_REFERENCE_OUTPUT")
	if root == "" || output == "" {
		t.Fatal("set HALO_FILM_CORPUS and HALO_REFERENCE_OUTPUT")
	}
	out, e := os.Create(output)
	if e != nil {
		t.Fatal(e)
	}
	defer out.Close()
	z := zlib.NewWriter(out)
	defer z.Close()
	enc := json.NewEncoder(z)
	totals := map[int]int{}
	films := 0
	e = filepath.WalkDir(root, func(path string, entry os.DirEntry, e error) error {
		if e != nil {
			return e
		}
		if entry.IsDir() || entry.Name() != "film.json" {
			return nil
		}
		data, e := os.ReadFile(path)
		if e != nil {
			return e
		}
		var manifest struct {
			Chunks []struct {
				File             string
				Index, ChunkType int
				Start            int `json:"start_time_offset_ms"`
				Kind             int `json:"chunk_type"`
			}
		}
		if e = json.Unmarshal(data, &manifest); e != nil {
			return e
		}
		folder := filepath.Dir(path)
		chunks := source.MemoryChunks{}
		meta := []types.ChunkMeta{}
		var bootstrap []byte
		for _, c := range manifest.Chunks {
			b, e := os.ReadFile(filepath.Join(folder, c.File))
			if e != nil {
				return e
			}
			chunks = append(chunks, b)
			meta = append(meta, types.ChunkMeta{Index: c.Index, ChunkType: c.Kind, StartMS: c.Start})
			if c.Kind == 1 {
				bootstrap = b
			}
		}
		film, e := source.Load(chunks, meta)
		if e != nil {
			return e
		}
		fc := NewFilmContext(film)
		cfg := fc.CadreDeBalayage()
		reg, e := ParseRegistryChunk(bootstrap)
		if e != nil {
			return e
		}
		world := NewWorld(reg)
		lastContext := ""
		ctx := ContexteParDefaut()
		ctx.Profil = cfg.Profil
		for ci, c := range manifest.Chunks {
			if c.Kind != 2 {
				continue
			}
			world.PoserChunkCourant(c.Index)
			rel, e := filepath.Rel(root, filepath.Join(folder, c.File))
			if e != nil {
				return e
			}
			for _, pk := range WalkPackets(chunks[ci]) {
				pay := pk.Payload(chunks[ci])
				if pk.Type == PacketTypeKeyframe {
					records, _ := WalkKeyframeRecords(pay, reg, ctx)
					for _, r := range records {
						if !r.SansArchetype() {
							world.BindImageCle(uint32(r.Gen), uint32(r.Slot), uint32(r.Archetype))
						}
					}
				} else if pk.Type == PacketTypeDelta {
					before, _ := json.Marshal(world.slots)
					contextBytes, _ := json.Marshal(map[string]any{"slots": json.RawMessage(before), "namespace": world.nsImageCle})
					var context any
					if string(contextBytes) != lastContext {
						context = json.RawMessage(contextBytes)
						lastContext = string(contextBytes)
					}
					records, views, end := DecodeFrameViewsCurseur(pay, world, cfg, 3, 2)
					var runtimeComponents []map[string]any
					for recordIndex, record := range records {
						for _, component := range record.Trace.Comps {
							if component.Name == compVehicleTypePhysics {
								runtimeComponents = append(runtimeComponents, map[string]any{
									"record": recordIndex, "name": component.Name,
									"start_bit": component.StartBit,
								})
							}
						}
					}
					totals[views]++
					if e := enc.Encode(map[string]any{"file": rel, "offset": pk.Start, "views": views, "end": end, "context": context, "runtime_components": runtimeComponents}); e != nil {
						return e
					}
				}
			}
		}
		films++
		return nil
	})
	if e != nil {
		t.Fatal(e)
	}
	t.Logf("films=%d totals=%v", films, totals)
	if films != 32 {
		t.Fatal(films)
	}
}
