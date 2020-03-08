#include "lib.hpp"

#include <atomic>
#include <cstdlib>
#include <cstring>

struct Buffer {
    explicit Buffer(const char* data) : _data(data) {}
    ~Buffer() {
        free((void*)_data);
    }
    const char* _data;
};

#ifdef __APPLE__
Buffer* CopyString(CFStringRef s) {
    CFIndex length = CFStringGetLength(s);
    CFIndex maxSize = CFStringGetMaximumSizeForEncoding(length, kCFStringEncodingUTF8) + 1;
    char* data = (char*)malloc(maxSize);
    CFStringGetCString(s, data, maxSize, kCFStringEncodingUTF8);
    return new Buffer(data);
}
#else
Buffer* CopyString(const char* s) {
    size_t l = strlen(s);
    char* buf = (char*)malloc(l);
    memcpy(buf, s, l);
    return new Buffer(buf);
}
#endif

struct StringArg {
    explicit StringArg(Buffer** dest) : _temp(nullptr), _dest(dest) {}

#ifdef __APPLE__
    ~StringArg() {
        if (_temp == nullptr) {
            *_dest = nullptr;
        } else {
            CFIndex length = CFStringGetLength(_temp);
            CFIndex maxSize = CFStringGetMaximumSizeForEncoding(length, kCFStringEncodingUTF8) + 1;
            char* data = (char*)malloc(maxSize);
            CFStringGetCString(_temp, data, maxSize, kCFStringEncodingUTF8);
            *_dest = new Buffer(data);
            CFRelease(_temp);
        }
    }

    operator CFStringRef*() {
        return &_temp;
    }

    CFStringRef _temp;
#else
    ~StringArg() {
        *_dest = new Buffer(_temp);
    }

    operator const char**() {
        return &_temp;
    }

    const char* _temp;
#endif

    Buffer** _dest;
};

#ifdef __APPLE__
CFStringRef CStringToString(const char* s) {
    return CFStringCreateWithCString(nullptr, s, kCFStringEncodingUTF8);
}

struct StringToCString {
    explicit StringToCString(CFStringRef s) {
        CFIndex length = CFStringGetLength(s);
        CFIndex maxSize = CFStringGetMaximumSizeForEncoding(length, kCFStringEncodingUTF8) + 1;
        _buffer = (char*)malloc(maxSize);
        CFStringGetCString(s, _buffer, maxSize, kCFStringEncodingUTF8);
    }

    ~StringToCString() {
        free(_buffer);
    }

    operator const char*() const {
        return _buffer;
    }

    char* _buffer;
};

typedef CFStringRef String;
#else
const char* CStringToString(const char* s) {
    return s;
}

const char* StringToCString(const char* s) {
    return s;
}

typedef const char* String;
#endif

extern "C" {

ULONG blackmagic_raw_unknown_add_ref(IUnknown* obj) {
    return obj->AddRef();
}

ULONG blackmagic_raw_unknown_release(IUnknown* obj) {
    return obj->Release();
}

HRESULT blackmagic_raw_unknown_query_interface(IUnknown* obj, REFIID iid, LPVOID* iface) {
    return obj->QueryInterface(iid, iface);
}

IBlackmagicRawFactory* create_blackmagic_raw_factory_instance_from_path(const char* path) {
    return CreateBlackmagicRawFactoryInstanceFromPath(CStringToString(path));
}

HRESULT blackmagic_raw_factory_create_codec(IBlackmagicRawFactory* factory, IBlackmagicRaw** out) {
    return factory->CreateCodec(out);
}

HRESULT blackmagic_raw_open_clip(IBlackmagicRaw* codec, const char* fileName, IBlackmagicRawClip** out) {
    return codec->OpenClip(CStringToString(fileName), out);
}

HRESULT blackmagic_raw_set_callback(IBlackmagicRaw* codec, IBlackmagicRawCallback* callback) {
    return codec->SetCallback(callback);
}

HRESULT blackmagic_raw_flush_jobs(IBlackmagicRaw* codec) {
    return codec->FlushJobs();
}

HRESULT blackmagic_raw_clip_get_width(IBlackmagicRawClip* clip, uint32_t *out) {
    return clip->GetWidth(out);
}

HRESULT blackmagic_raw_clip_get_height(IBlackmagicRawClip* clip, uint32_t *out) {
    return clip->GetHeight(out);
}

HRESULT blackmagic_raw_clip_get_frame_rate(IBlackmagicRawClip* clip, float *out) {
    return clip->GetFrameRate(out);
}

HRESULT blackmagic_raw_clip_get_frame_count(IBlackmagicRawClip* clip, uint64_t *out) {
    return clip->GetFrameCount(out);
}

HRESULT blackmagic_raw_clip_get_metadata_iterator(IBlackmagicRawClip* clip, IBlackmagicRawMetadataIterator** iterator) {
    return clip->GetMetadataIterator(iterator);
}

HRESULT blackmagic_raw_clip_create_job_read_frame(IBlackmagicRawClip* clip, uint64_t frameIndex, IBlackmagicRawJob** job) {
    return clip->CreateJobReadFrame(frameIndex, job);
}

HRESULT blackmagic_raw_clip_create_job_trim(IBlackmagicRawClip* clip, const char* fileName, uint64_t frameIndex, uint64_t frameCount, IBlackmagicRawClipProcessingAttributes* clipProcessingAttributes, IBlackmagicRawFrameProcessingAttributes* frameProcessingAttributes, IBlackmagicRawJob** job) {
    return clip->CreateJobTrim(CStringToString(fileName), frameIndex, frameCount, clipProcessingAttributes, frameProcessingAttributes, job);
}

HRESULT blackmagic_raw_clip_audio_get_channel_count(IBlackmagicRawClipAudio* audio, uint32_t *out) {
    return audio->GetAudioChannelCount(out);
}

HRESULT blackmagic_raw_clip_audio_get_sample_rate(IBlackmagicRawClipAudio* audio, uint32_t *out) {
    return audio->GetAudioSampleRate(out);
}

HRESULT blackmagic_raw_clip_audio_get_sample_count(IBlackmagicRawClipAudio* audio, uint64_t *out) {
    return audio->GetAudioSampleCount(out);
}

HRESULT blackmagic_raw_metadata_iterator_next(IBlackmagicRawMetadataIterator* it) {
    return it->Next();
}

HRESULT blackmagic_raw_metadata_iterator_get_key(IBlackmagicRawMetadataIterator* it, Buffer** key) {
    return it->GetKey(StringArg(key));
}

HRESULT blackmagic_raw_metadata_iterator_get_data(IBlackmagicRawMetadataIterator* it, Variant* data) {
    return it->GetData(data);
}

HRESULT blackmagic_raw_job_submit(IBlackmagicRawJob* job) {
    return job->Submit();
}

HRESULT blackmagic_raw_frame_set_resource_format(IBlackmagicRawFrame* frame, BlackmagicRawResourceFormat format) {
    return frame->SetResourceFormat(format);
}

HRESULT blackmagic_raw_frame_create_job_decode_and_process_frame(IBlackmagicRawFrame* frame, IBlackmagicRawClipProcessingAttributes* clipProcessingAttributes, IBlackmagicRawFrameProcessingAttributes* frameProcessingAttributes, IBlackmagicRawJob** job) {
    return frame->CreateJobDecodeAndProcessFrame(clipProcessingAttributes, frameProcessingAttributes, job);
}

HRESULT blackmagic_raw_processed_image_get_width(IBlackmagicRawProcessedImage* img, uint32_t* out) {
    return img->GetWidth(out);
}

HRESULT blackmagic_raw_processed_image_get_height(IBlackmagicRawProcessedImage* img, uint32_t* out) {
    return img->GetHeight(out);
}

HRESULT blackmagic_raw_processed_image_get_resource_size_bytes(IBlackmagicRawProcessedImage* img, uint32_t* out) {
    return img->GetResourceSizeBytes(out);
}

HRESULT blackmagic_raw_processed_image_get_resource(IBlackmagicRawProcessedImage* img, void** bytes) {
    return img->GetResource(bytes);
}

extern void callback_read_complete(void* impl, IBlackmagicRawJob* job, HRESULT result, IBlackmagicRawFrame* frame);
extern void callback_decode_complete(void* impl, IBlackmagicRawJob* job, HRESULT result);
extern void callback_process_complete(void* impl, IBlackmagicRawJob* job, HRESULT result, IBlackmagicRawProcessedImage* processedImage);
extern void callback_trim_progress(void* impl, IBlackmagicRawJob* job, float progress);
extern void callback_trim_complete(void* impl, IBlackmagicRawJob* job, HRESULT result);
extern void callback_sidecar_metadata_parse_warning(void* impl, IBlackmagicRawClip* clip, const char* fileName, uint32_t lineNumber, const char* info);
extern void callback_sidecar_metadata_parse_error(void* impl, IBlackmagicRawClip* clip, const char* fileName, uint32_t lineNumber, const char* info);
extern void callback_prepare_pipeline_complete(void* impl, void* userData, HRESULT result);

struct Callback: IBlackmagicRawCallback {
    explicit Callback(void* implementation) : _ref_count(1), _implementation(implementation) {}
    virtual ~Callback() {}

    virtual void ReadComplete(IBlackmagicRawJob* job, HRESULT result, IBlackmagicRawFrame* frame) {
        callback_read_complete(_implementation, job, result, frame);
    }

    virtual void DecodeComplete(IBlackmagicRawJob* job, HRESULT result) {
        callback_decode_complete(_implementation, job, result);
    }

    virtual void ProcessComplete(IBlackmagicRawJob* job, HRESULT result, IBlackmagicRawProcessedImage* processedImage) {
        callback_process_complete(_implementation, job, result, processedImage);
    }

    virtual void TrimProgress(IBlackmagicRawJob* job, float progress) {
        callback_trim_progress(_implementation, job, progress);
    }

    virtual void TrimComplete(IBlackmagicRawJob* job, HRESULT result) {
        callback_trim_complete(_implementation, job, result);
    }

    virtual void SidecarMetadataParseWarning(IBlackmagicRawClip* clip, String fileName, uint32_t lineNumber, String info) {
        callback_sidecar_metadata_parse_warning(_implementation, clip, StringToCString(fileName), lineNumber, StringToCString(info));
    }

    virtual void SidecarMetadataParseError(IBlackmagicRawClip* clip, String fileName, uint32_t lineNumber, String info) {
        callback_sidecar_metadata_parse_error(_implementation, clip, StringToCString(fileName), lineNumber, StringToCString(info));
    }

    virtual void PreparePipelineComplete(void* userData, HRESULT result) {
        callback_prepare_pipeline_complete(_implementation, userData, result);
    }

    virtual HRESULT QueryInterface(REFIID iid, LPVOID *ppv) {
        if (ppv == NULL) {
            return E_INVALIDARG;
        }

        *ppv = NULL;

        CFUUIDBytes iunknown = CFUUIDGetUUIDBytes(IUnknownUUID);
        HRESULT result = E_NOINTERFACE;
        if (memcmp(&iid, &iunknown, sizeof(REFIID)) == 0) {
            *ppv = this;
            AddRef();
            result = S_OK;
        } else if (memcmp(&iid, &IID_IBlackmagicRawCallback, sizeof(REFIID)) == 0) {
            *ppv = (IBlackmagicRawCallback*)this;
            AddRef();
            result = S_OK;
        }

        return result;
    }

    virtual ULONG AddRef() {
        return _ref_count.fetch_add(1);
    }

    virtual ULONG Release() {
        int refs = _ref_count.fetch_sub(1);
        if (refs == 0) {
            delete this;
        }
        return refs;
    }

    std::atomic<uint32_t> _ref_count;
    void* _implementation;
};

IBlackmagicRawCallback* create_blackmagic_raw_callback(void* implementation) {
    return new Callback(implementation);
}

const void* buffer_data(Buffer* buf) {
    return buf->_data;
}

void buffer_release(Buffer* obj) {
    if (obj != nullptr) {
        delete obj;
    }
}

void blackmagic_raw_variant_get_string(Variant* v, Buffer** out) {
    *out = CopyString(v->bstrVal);
}

}
