vim.api.nvim_create_user_command('Json',function()
    vim.api.nvim_command("%!python3 -m json.tool")
end,{})
