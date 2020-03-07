#include <BlackmagicRawAPI.h>

extern "C" {

ULONG blackmagic_raw_unknown_add_ref(IUnknown* obj);
ULONG blackmagic_raw_unknown_release(IUnknown* obj);
HRESULT blackmagic_raw_unknown_query_interface(IUnknown* obj, REFIID iid, LPVOID* iface);

IBlackmagicRawFactory* create_blackmagic_raw_factory_instance_from_path(const char* path);
HRESULT blackmagic_raw_factory_create_codec(IBlackmagicRawFactory* factory, IBlackmagicRaw** out);

HRESULT blackmagic_raw_open_clip(IBlackmagicRaw* codec, const char* fileName, IBlackmagicRawClip** out);
HRESULT blackmagic_raw_set_callback(IBlackmagicRaw* codec, IBlackmagicRawCallback* callback);
HRESULT blackmagic_raw_flush_jobs(IBlackmagicRaw* codec);

HRESULT blackmagic_raw_clip_get_width(IBlackmagicRawClip* clip, uint32_t *out);
HRESULT blackmagic_raw_clip_get_height(IBlackmagicRawClip* clip, uint32_t *out);
HRESULT blackmagic_raw_clip_get_frame_rate(IBlackmagicRawClip* clip, float *out);
HRESULT blackmagic_raw_clip_get_frame_count(IBlackmagicRawClip* clip, uint64_t *out);
HRESULT blackmagic_raw_clip_create_job_read_frame(IBlackmagicRawClip* clip, uint64_t frameIndex, IBlackmagicRawJob** job);
HRESULT blackmagic_raw_clip_create_job_trim(IBlackmagicRawClip* clip, const char* fileName, uint64_t frameIndex, uint64_t frameCount, IBlackmagicRawClipProcessingAttributes* clipProcessingAttributes, IBlackmagicRawFrameProcessingAttributes* frameProcessingAttributes, IBlackmagicRawJob** job);

HRESULT blackmagic_raw_clip_audio_get_channel_count(IBlackmagicRawClipAudio* audio, uint32_t *out);
HRESULT blackmagic_raw_clip_audio_get_sample_rate(IBlackmagicRawClipAudio* audio, uint32_t *out);
HRESULT blackmagic_raw_clip_audio_get_sample_count(IBlackmagicRawClipAudio* audio, uint64_t *out);

HRESULT blackmagic_raw_job_submit(IBlackmagicRawJob* job);

HRESULT blackmagic_raw_frame_set_resource_format(IBlackmagicRawFrame* frame, BlackmagicRawResourceFormat format);
HRESULT blackmagic_raw_frame_create_job_decode_and_process_frame(IBlackmagicRawFrame* frame, IBlackmagicRawClipProcessingAttributes* clipProcessingAttributes, IBlackmagicRawFrameProcessingAttributes* frameProcessingAttributes, IBlackmagicRawJob** job);

HRESULT blackmagic_raw_processed_image_get_width(IBlackmagicRawProcessedImage* img, uint32_t* out);
HRESULT blackmagic_raw_processed_image_get_height(IBlackmagicRawProcessedImage* img, uint32_t* out);
HRESULT blackmagic_raw_processed_image_get_resource_size_bytes(IBlackmagicRawProcessedImage* img, uint32_t* out);
HRESULT blackmagic_raw_processed_image_get_resource(IBlackmagicRawProcessedImage* img, void** bytes);

IBlackmagicRawCallback* create_blackmagic_raw_callback(void* implementation);

}
