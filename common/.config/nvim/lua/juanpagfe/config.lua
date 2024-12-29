-- It is a general use file to save data to use on my nvim setup
local cfgPath = vim.fn.stdpath('data') .. '/jpconfig.json'

local M = {}

local function get_cfg()
    local file = io.open(cfgPath, "r")
    if file then
        local cfgStr = file:read("*all")
        local cfg = (not cfgStr or string.len(cfgStr) == 0) and {} or vim.json.decode(cfgStr)
        file:close()
        return cfg
    else
        return nil;
    end
end

local function save_cfg(cfg)
    local file = io.open(cfgPath, "w")
    if file then
        file:write(vim.json.encode(cfg))
        file:close()
        return cfg
    else
        print("Failed to open config file for writing")
        return nil;
    end
end

function M.set_cfg_val(key, value)
    local cfg = get_cfg()
    if cfg == nil then
        cfg = {}
    end
    cfg[key] = value
    save_cfg(cfg)
end

function M.get_cfg_val(key, default)
    default = default or ""
    local cfg = get_cfg()
    if cfg == nil then
        cfg = {}
        cfg[key] = ""
        save_cfg(cfg)
        return ""
    else
        local value = cfg[key]
        if value == nil then
            cfg[key] = default
            save_cfg(cfg)
            return default
        end
        return value
    end
end

return M
