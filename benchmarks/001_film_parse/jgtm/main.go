// This helper measures JGtm's public loading, registry, and highlight APIs.
// Its full replay builder needs an external map-bounds catalogue, so these
// timings are a partial decode and must be labeled as such in comparisons.
package main

import (
	"archive/tar"
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"sort"
	"time"

	decfilm "github.com/JGtm/halo-infinite-film-decoder/decfilm"
)

type entry struct {
	Kind    int    `json:"kind"`
	Index   int    `json:"index"`
	StartMS int    `json:"start_ms"`
	File    string `json:"file"`
}

type manifest struct {
	Chunks []entry `json:"chunks"`
}

func loadFixture(path string) (decfilm.MemoryChunks, []decfilm.ChunkMeta, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, nil, err
	}
	defer f.Close()

	files := make(map[string][]byte)
	reader := tar.NewReader(f)
	for {
		hdr, err := reader.Next()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return nil, nil, err
		}
		if hdr.Typeflag != tar.TypeReg && hdr.Typeflag != tar.TypeRegA {
			continue
		}
		data, err := io.ReadAll(reader)
		if err != nil {
			return nil, nil, err
		}
		if _, exists := files[hdr.Name]; exists {
			return nil, nil, fmt.Errorf("duplicate archive member %q", hdr.Name)
		}
		files[hdr.Name] = data
	}
	var m manifest
	if err := json.Unmarshal(files["manifest.json"], &m); err != nil {
		return nil, nil, err
	}
	if len(m.Chunks) == 0 {
		return nil, nil, errors.New("empty film manifest")
	}
	sort.Slice(m.Chunks, func(i, j int) bool { return m.Chunks[i].Index < m.Chunks[j].Index })
	chunks := make(decfilm.MemoryChunks, 0, len(m.Chunks))
	meta := make([]decfilm.ChunkMeta, 0, len(m.Chunks))
	for _, e := range m.Chunks {
		data, ok := files[e.File]
		if !ok {
			return nil, nil, fmt.Errorf("missing chunk %q", e.File)
		}
		chunks = append(chunks, data)
		meta = append(meta, decfilm.ChunkMeta{Index: e.Index, ChunkType: e.Kind, StartMS: e.StartMS})
	}
	if meta[0].ChunkType != 1 || meta[len(meta)-1].ChunkType != 3 {
		return nil, nil, errors.New("missing registry or summary chunk")
	}
	return chunks, meta, nil
}

var observed int

func parse(chunks decfilm.MemoryChunks, meta []decfilm.ChunkMeta) error {
	owned := make(decfilm.MemoryChunks, len(chunks))
	for i, chunk := range chunks {
		owned[i] = append([]byte(nil), chunk...)
	}
	film, err := decfilm.Load(owned, meta)
	if err != nil {
		return err
	}
	registry, err := decfilm.ParseRegistryChunk(film.Chunk(0))
	if err != nil {
		return err
	}
	majorVersion, ok := decfilm.FilmMajorVersionFromHeader(film.Chunk(0))
	if !ok {
		return errors.New("missing film major version")
	}
	highlights, err := decfilm.ParseHighlightEvents(film.Chunk(len(chunks)-1), majorVersion)
	if err != nil {
		return err
	}
	observed = len(film.AllPackets()) + len(highlights)
	_ = registry
	return nil
}

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "usage: jgtm-bench fixture.tar")
		os.Exit(2)
	}
	chunks, meta, err := loadFixture(os.Args[1])
	if err == nil {
		err = parse(chunks, meta) // warm up and reject unsupported input
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	out := bufio.NewWriter(os.Stdout)
	fmt.Fprintln(out, "ready")
	out.Flush()
	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		switch scanner.Text() {
		case "run":
			start := time.Now()
			err := parse(chunks, meta)
			if err != nil {
				fmt.Fprintln(os.Stderr, err)
				os.Exit(1)
			}
			fmt.Fprintf(out, "%.9f\n", float64(time.Since(start).Nanoseconds())/1e6)
			out.Flush()
		case "quit":
			return
		default:
			fmt.Fprintln(os.Stderr, "unknown benchmark command")
			os.Exit(2)
		}
	}
	if err := scanner.Err(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
