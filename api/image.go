package api

import (
	"bytes"
	"io"
	"net/http"
	"strconv"
	"strings"
	"uzume_backend/helper"
	"uzume_backend/model"

	"github.com/labstack/echo"
)

type ImageResponse struct {
	ImageId string   `json:"image_id"`
	Tags    []string `json:"tags"`
}

func QueryParamToArray(str string) []string {
	var arr []string
	for _, tag := range strings.Split(str, ",") {
		if len(tag) > 0 {
			arr = append(arr, tag)
		}
	}

	return arr
}

func GetImages() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		tag_search_type := c.QueryParam("tag_search_type")
		var tag_list = QueryParamToArray(c.QueryParam("tags"))
		var page_str = c.QueryParam("page")
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		page, err := strconv.Atoi(page_str)
		if err != nil || page < 1 {
			page = 1
		}

		images, err := model.SearchImages(workspace, tag_list, tag_search_type, page)
		if err != nil {
			if err.Error() == "Unknown search type." {
				return c.JSON(http.StatusBadRequest, helper.ErrorMessage{ErrorMessage: err.Error()})
			}
			return err
		}

		return c.JSON(http.StatusOK, struct {
			Page   int            `json:"page"`
			Images []*model.Image `json:"images"`
		}{
			Page:   page,
			Images: images,
		})
	}
}

func GetGroupImages() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		group_id := c.Param("id")
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		images, err := model.GetGroupImages(workspace, group_id)
		if err != nil {
			if err.Error() == "group_id Not Found" {
				return c.JSON(http.StatusNotFound, helper.ErrorMessage{ErrorMessage: err.Error()})
			}
			return err
		}

		return c.JSON(http.StatusOK, struct {
			Images []*model.Image `json:"images"`
		}{
			Images: images,
		})
	}
}

func PatchImages() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		image_id := c.Param("id")
		param := new(struct {
			Author string `json:"author"`
			Memo   string `json:"memo"`
		})
		if err := c.Bind(param); err != nil {
			return err
		}
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		image, err := model.FindImageById(workspace, image_id)
		if err != nil {
			return err
		}
		image.Author = param.Author
		image.Memo = param.Memo
		image.Save()

		return c.JSON(http.StatusOK, image)
	}
}

func GetImageFile() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		image_id := c.Param("id")
		image_size := c.QueryParam("image_size")
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		image, err := model.FindImageById(workspace, image_id)
		if err != nil {
			return err
		}

		option := ""
		switch image_size {
		case "thumbnail":
			option = "thumb"
		case "original":
			option = ""
		default:
			return c.JSON(http.StatusBadRequest, helper.ErrorMessage{ErrorMessage: "invalid option."})
		}

		return c.File(image.ImagePath(option))
	}
}

func PostImages() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		image_file, err := c.FormFile("image")
		if err != nil {
			return err
		}
		author := c.FormValue("author")
		memo := c.FormValue("memo")
		var tag_list = QueryParamToArray(c.FormValue("tags"))
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		image := model.NewImage(workspace)

		src, err := image_file.Open()
		if err != nil {
			return err
		}
		defer src.Close()
		image_buffer := new(bytes.Buffer)
		if _, err := io.Copy(image_buffer, src); err != nil {
			return err
		}
		if err := image.CreateImageAndSave(image_file.Filename, image_buffer); err != nil {
			return err
		}

		for _, tag := range tag_list {
			if err := image.AddTag(tag); err != nil {
				if err.Error() == "invalid tag_id" {
					return c.JSON(http.StatusBadRequest, helper.ErrorMessage{ErrorMessage: err.Error()})
				}
				return err
			}
		}
		if err := image.Save(); err != nil {
			return err
		}

		image.Memo = memo
		image.Author = author
		if err := image.Save(); err != nil {
			return err
		}

		if len(tag_list) == 0 {
			if err := image.AddTag(model.SYSTEM_TAG_UNCATEGORIZED); err != nil {
				return err
			}
			if err := image.Save(); err != nil {
				return err
			}
		}

		return c.JSON(http.StatusCreated, struct {
			ImageId string `json:"image_id"`
		}{
			ImageId: image.Id,
		})
	}
}

func PatchImageTag() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		image_id := c.Param("id")
		param := new(struct {
			TagId string `json:"tag_id"`
		})
		if err := c.Bind(param); err != nil {
			return err
		}
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		image, err := model.FindImageById(workspace, image_id)
		if err != nil {
			return err
		}

		if err := image.AddTag(param.TagId); err != nil {
			if err.Error() == "invalid tag_id" || err.Error() == "このタグは既に登録されています" {
				return c.JSON(http.StatusBadRequest, helper.ErrorMessage{ErrorMessage: err.Error()})
			}
			return err
		}
		if err := image.Save(); err != nil {
			return err
		}

		return c.JSON(http.StatusNoContent, "")
	}
}

func DeleteImageTag() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		image_id := c.Param("image_id")
		tag_id := c.Param("tag_id")
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		image, err := model.FindImageById(workspace, image_id)
		if err != nil {
			return err
		}
		image.RemoveTag(tag_id)
		if err := image.Save(); err != nil {
			return err
		}

		return c.JSON(http.StatusNoContent, "")
	}
}

func PostImagesGroup() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		param := new(struct {
			Images string `json:"images"`
		})
		if err := c.Bind(param); err != nil {
			return err
		}
		image_id_list := QueryParamToArray(param.Images)
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		var image_list []*model.Image = nil
		for _, image_id := range image_id_list {
			image, err := model.FindImageById(workspace, image_id)
			if err != nil {
				return err
			}
			image_list = append(image_list, image)
		}

		if err := model.GroupingImages(image_list); err != nil {
			return err
		}
		// TODO: トランザクション作ってロールバックできるようにする？
		for _, image := range image_list {
			if err := image.Save(); err != nil {
				return err
			}
		}

		return c.JSON(http.StatusCreated, struct {
			Images []*model.Image `json:"images"`
		}{
			Images: image_list,
		})
	}
}

func DeleteImagesGroup() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		group_id := c.Param("id")
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		if err := model.DeleteGroupAndSave(workspace, group_id); err != nil {
			return err
		}

		return c.JSON(http.StatusOK, "")
	}
}

func PatchImagesGroupSort() echo.HandlerFunc {
	return func(c echo.Context) (err error) {
		param := new(struct {
			Images []string `json:"images"`
		})
		if err := c.Bind(param); err != nil {
			return err
		}
		image_id_list := param.Images
		workspace_id := helper.LoggedinWrokspaceId(c)

		workspace, err := model.FindWorkspaceById(workspace_id)
		if err != nil {
			return err
		}

		// 最初の一枚からgroup_idを割り出す
		image, err := model.FindImageById(workspace, image_id_list[0])
		if err != nil {
			return err
		}
		group_id := image.GroupId

		images, err := model.GetGroupImages(workspace, group_id)
		if err != nil {
			if err.Error() == "group_id Not Found" {
				return c.JSON(http.StatusNotFound, helper.ErrorMessage{ErrorMessage: err.Error()})
			}
			return err
		}

		// 遅いようなら連想配列を使うなどして高速化を検討する
		for i, image_id := range image_id_list {
			for _, image := range images {
				if image.Id == image_id {
					image.SortOfGroup = i + 1
					image.Save()
				}
			}
		}

		return c.JSON(http.StatusOK, "")
	}
}
