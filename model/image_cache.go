package model

import (
	"fmt"
	"io/ioutil"
	"strings"
)

type imageCache struct {
	SortedImages        []*Image
	GroupedSortedImages []*Image
	IdToImages          map[string]*Image   // map[image_id]
	GroupIdToImages     map[string][]*Image // map[group_id]
	TagToImages         map[string][]*Image // map[tag_id]
}

var g_image_cache = make(map[string]*imageCache) // map[workspace_id]

func ResetImageCache() {
	g_image_cache = make(map[string]*imageCache)
}

func getImageCache(workspace_id string) *imageCache {
	if g_image_cache[workspace_id] == nil {
		g_image_cache[workspace_id] = new(imageCache)
	}

	if g_image_cache[workspace_id].IdToImages == nil {
		g_image_cache[workspace_id].IdToImages = make(map[string]*Image)
	}

	if g_image_cache[workspace_id].TagToImages == nil {
		g_image_cache[workspace_id].TagToImages = make(map[string][]*Image)
	}

	return g_image_cache[workspace_id]
}

func isImageCacheExist(workspace *Workspace) bool {
	_, cache_exists := g_image_cache[workspace.Id]
	return cache_exists
}

func getAllImageCache(workspace *Workspace) ([]*Image, error) {
	if !isImageCacheExist(workspace) {
		if err := refleshImageCache(workspace); err != nil {
			return nil, err
		}
	}

	if _, ok := g_image_cache[workspace.Id]; !ok {
		return []*Image{}, nil
	}

	return g_image_cache[workspace.Id].GroupedSortedImages, nil
}

func getImageCacheByTagId(workspace *Workspace, tag_id string) ([]*Image, error) {
	if !isImageCacheExist(workspace) {
		if err := refleshImageCache(workspace); err != nil {
			return nil, err
		}
	}

	if _, ok := g_image_cache[workspace.Id]; !ok {
		return []*Image{}, nil
	}

	return g_image_cache[workspace.Id].TagToImages[tag_id], nil
}

func getImageCacheByGroupId(workspace *Workspace, group_id string) ([]*Image, error) {
	if !isImageCacheExist(workspace) {
		if err := refleshImageCache(workspace); err != nil {
			return nil, err
		}
	}

	if _, ok := g_image_cache[workspace.Id]; !ok {
		return []*Image{}, nil
	}

	return g_image_cache[workspace.Id].GroupIdToImages[group_id], nil
}

func createImageCache(image *Image) {
	if !isImageCacheExist(image.Workspace) {
		if err := refleshImageCache(image.Workspace); err != nil {
			// 失敗した場合は、一旦キャッシュを放棄する
			destroyImageCache(image.Workspace)
			return
		}
	}

	image_cache := getImageCache(image.Workspace.Id)
	image_cache.IdToImages[image.Id] = image
	for _, tag_id := range image.Tags {
		image_cache.TagToImages[tag_id] = append(image_cache.TagToImages[tag_id], image)
	}

	resetSortedImages(image.Workspace)
}

func updateImageCache(next_image *Image, prev_image *Image) {
	if !isImageCacheExist(next_image.Workspace) {
		if err := refleshImageCache(next_image.Workspace); err != nil {
			// 失敗した場合は、一旦キャッシュを放棄する
			destroyImageCache(next_image.Workspace)
			return
		}
	}

	image_cache := getImageCache(next_image.Workspace.Id)

	// image自体の内容を更新
	*(image_cache.IdToImages[next_image.Id]) = *next_image
	image := image_cache.IdToImages[next_image.Id]

	// 増えたタグについての処理
	for _, next_tag_id := range image.Tags {
		is_added := true
		for _, prev_tag_id := range prev_image.Tags {
			if next_tag_id == prev_tag_id {
				is_added = false
				break
			}
		}

		if is_added {
			image_cache.TagToImages[next_tag_id] = append(image_cache.TagToImages[next_tag_id], image)
		}
	}

	// 消えたタグについての処理
	for _, prev_tag_id := range prev_image.Tags {
		is_deleted := true
		for _, next_tag_id := range image.Tags {
			if next_tag_id == prev_tag_id {
				is_deleted = false
				break
			}
		}

		if is_deleted {
			var new_images []*Image
			for _, img := range image_cache.TagToImages[prev_tag_id] {
				if image.Id != img.Id {
					new_images = append(new_images, img)
				}
			}
			image_cache.TagToImages[prev_tag_id] = new_images
		}
	}

	// 紐づく画像が0になったkey(タグ)を削除する
	new_tag_to_images := make(map[string][]*Image)
	for tag_id, images := range image_cache.TagToImages {
		if len(images) > 0 {
			new_tag_to_images[tag_id] = images
		}
	}
	image_cache.TagToImages = new_tag_to_images

	if image.GroupId != prev_image.GroupId {
		// 古い方のグループから削除
		if prev_image.GroupId != "" {
			var new_group_to_images []*Image = nil
			for _, img := range image_cache.GroupIdToImages[prev_image.GroupId] {
				if img.GroupId != prev_image.GroupId {
					new_group_to_images = append(new_group_to_images, image)
				}
			}
			image_cache.GroupIdToImages[prev_image.GroupId] = new_group_to_images
		}
		// 新しい方のグループに追加
		if image.GroupId != "" {
			image_cache.GroupIdToImages[image.GroupId] = append(image_cache.GroupIdToImages[image.GroupId], image)
		}

		resetSortedImages(image.Workspace)
	}

	// グループthumbが変化したとき
	if image.IsGroupThumbNail != prev_image.IsGroupThumbNail {
		resetSortedImages(image.Workspace)
		if image.IsGroupThumbNail {
			// groupedリストに追加
			if !isExistCacheInGroupedSortedImages(image) {
				image_cache.GroupedSortedImages = append(image_cache.GroupedSortedImages, image)
			}
		}

		if !image.IsGroupThumbNail {
			// grouped リストから削除
			var new_grouped_sorted_images []*Image = nil
			for _, img := range image_cache.GroupedSortedImages {
				if img.Id != image.Id {
					new_grouped_sorted_images = append(new_grouped_sorted_images, image)
				}
			}
			image_cache.GroupedSortedImages = new_grouped_sorted_images
		}
	}
}

func isExistCacheInGroupedSortedImages(image *Image) bool {
	for _, img := range g_image_cache[image.Workspace.Id].GroupedSortedImages {
		if img.Id == image.Id {
			return true
		}
	}
	return false
}

func resetSortedImages(workspace *Workspace) error {
	g_image_cache[workspace.Id].SortedImages = make([]*Image, len(g_image_cache[workspace.Id].IdToImages))

	var i = 0 // TODO: このiは_と同義では？
	for _, v := range g_image_cache[workspace.Id].IdToImages {
		g_image_cache[workspace.Id].SortedImages[i] = v
		i++
	}

	sortImageList(g_image_cache[workspace.Id].SortedImages)

	g_image_cache[workspace.Id].GroupedSortedImages = nil
	for _, v := range g_image_cache[workspace.Id].SortedImages {
		if v.GroupId == "" || v.IsGroupThumbNail {
			// group化されていないか、グループのサムネだったらリストに入れる
			g_image_cache[workspace.Id].GroupedSortedImages = append(g_image_cache[workspace.Id].GroupedSortedImages, v)
		}
	}

	return nil
}

func refleshImageCache(workspace *Workspace) error {
	image := NewImage(workspace)
	all_image_files, err := ioutil.ReadDir(image.ImagesDirPath())
	if err != nil {
		if strings.HasSuffix(err.Error(), "no such file or directory") {
			destroyImageCache(workspace)
			return nil
		}
		return err
	}

	cache := new(imageCache)
	cache.IdToImages = make(map[string]*Image)
	cache.GroupIdToImages = make(map[string][]*Image)
	cache.TagToImages = make(map[string][]*Image)

	for _, f := range all_image_files {
		file_name := f.Name()
		if !strings.HasSuffix(file_name, IMAGE_DIR_EXT) {
			continue
		}

		image_id := file_name[:len(file_name)-len(IMAGE_DIR_EXT)]
		image, err := FindImageById(workspace, image_id)
		if err != nil {
			return err
		}

		cache.IdToImages[image.Id] = image
		if image.GroupId != "" {
			cache.GroupIdToImages[image.GroupId] = append(cache.GroupIdToImages[image.GroupId], image)
		}
		for _, t := range image.Tags {
			cache.TagToImages[t] = append(cache.TagToImages[t], image)
		}
	}

	// TODO: IsGroupThumbNailフラグがおかしかった場合に調整する
	// for group_id, images := range cache.GroupIdToImages {
	// 	for image := range images {
	// 	}
	// }

	g_image_cache[workspace.Id] = cache
	resetSortedImages(workspace)

	return nil
}

func destroyImageCache(workspace *Workspace) {
	delete(g_image_cache, workspace.Id)
	fmt.Printf("[LOG] delete image cache workspace.Id=%s\n", workspace.Id)
}
